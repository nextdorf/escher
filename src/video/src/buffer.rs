use crate::ffi::{AVFrame, self};

use std::{sync::{Arc, Mutex, mpsc}};

use escher_schedule as schedule;

pub mod frame_protocol {
  use std::{sync::{Arc, Mutex}, fmt::Debug};
  pub use escher_schedule::RequestKind; 

  use crate::ffi;

  pub enum Request {
    SetSwsContext {new_width: i32, new_height: i32, new_pix_fmt: ffi::AVPixelFormat, width: i32, height: i32, pix_fmt: ffi::AVPixelFormat, flags: u32, param: Arc<Mutex<Vec<f64>>>},
    // Ping(usize),
  }
  // type RequestKind = escher_schedule::RequestKind;
  pub type Response = escher_schedule::Response<()>;

  impl Clone for Request {
    fn clone(&self) -> Self {
      match self {
        Self::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, flags, param } => Self::SetSwsContext { new_width: new_width.clone(), new_height: new_height.clone(), new_pix_fmt: new_pix_fmt.clone(), width: width.clone(), height: height.clone(), pix_fmt: pix_fmt.clone(), flags: flags.clone(), param: param.clone() },
      }
    }
  }

  impl Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        Self::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, flags, param } => 
          f.debug_struct("SetSwsContext").field("new_width", new_width).field("new_height", new_height).field("new_pix_fmt", new_pix_fmt).field("width", width).field("height", height).field("pix_fmt", pix_fmt).field("flags", flags).field("param", param).finish(),
      }
    }
}
}

pub use schedule::BroadcastKind;
pub type RequestError = schedule::RequestError<frame_protocol::Request>;

pub struct FrameBuffer {
  frames: Vec<Arc<Mutex<Option<AVFrame>>>>,
  scheduler: schedule::Scheduler<frame_protocol::Request, ()>
}

// type FrameBufferWorkerHandle = schedule::Worker<frame_protocol::Request, ()>;

// struct FrameBufferWorker {
//   thread_idx: usize,
//   rx: mpsc::Receiver<(schedule::RequestKind, frame_protocol::Request)>,
//   tx: mpsc::Sender<frame_protocol::Response>,
//   state: WorkerState,
// }
struct WorkerState {
  sws_ctx: *mut ffi::SwsContext,
}

impl FrameBuffer {
  pub fn new(&self, num_workers: usize, new_width: i32, new_height: i32, new_pix_fmt: ffi::AVPixelFormat, width: i32, height: i32, pix_fmt: ffi::AVPixelFormat, scaling: ffi::SWS_Scaling) -> Arc<FrameBuffer>
  {
    use frame_protocol::Request;
    let scheduler = schedule::Scheduler::new(num_workers, |_| WorkerState {sws_ctx: std::ptr::null_mut()});
    let (flags, param) = scaling.into();
    for i in 0..num_workers {
      let param = Arc::new(Mutex::new(param.clone()));
      scheduler.request(Request::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, flags, param }, BroadcastKind::Specific(i)).unwrap();
    }
    Arc::new(FrameBuffer { frames: Vec::default(), scheduler })
  }


  pub fn request(&self, request: frame_protocol::Request, kind: BroadcastKind) -> Result<(), RequestError> {
    self.scheduler.request(request, kind)
  }

  pub fn handle_respones(&mut self) {
    self.scheduler.handle_respones(Self::inner_handle_respones).unwrap()
  }

  fn inner_handle_respones(resp: frame_protocol::Response, _tx: &mpsc::Sender<(frame_protocol::RequestKind, frame_protocol::Request)>) -> Result<(), ()> {
    match resp {
      schedule::Response::Init => Ok(()),
      schedule::Response::Ok(()) => Ok(()),
      schedule::Response::Ready(_) => Ok(()),
    }
  }
}

// impl FrameBufferWorker {
//   pub fn new(thread_idx: usize, t_resp: mpsc::Sender<schedule::Response<()>>) -> (mpsc::Sender<(schedule::RequestKind, frame_protocol::Request)>, JoinHandle<()>) {
//     use frame_protocol::Response;
//     let (tx, rx) = mpsc::channel();
//     let t = std::thread::spawn(move || {
//       let (t_req, r_req) = mpsc::channel::<frame_protocol::Request>();
//       tx.send(t_req).unwrap();
//       t_resp.send(Response::Ok).unwrap();
//       let worker_state = WorkerState { sws_ctx: std::ptr::null_mut() };
//       let mut worker = Self { thread_idx, rx: r_req, tx: t_resp, state: worker_state};

//       loop {
//         match worker.rx.recv() {
//           Ok(req) => match worker.tx.send(worker.state.handle(req)) {
//             Ok(()) => {},
//             Err(e) => eprintln!("Response Error: {:?}", e)
//           },
//           Err(e) => eprintln!("Request Error: {:?}", e)
//         }
//       }
//     });
//     (rx.recv().unwrap(), t)
//   }

// }

// impl FrameBufferWorkerHandle {
//   pub fn send(&self, kind: frame_protocol::RequestKind, request: frame_protocol::Request) -> Result<(), RequestError> {
//     match self.tx.send((kind, request)) {
//       Ok(()) => Ok(()),
//       Err(e) => Err(e.into())
//     }
//   }

//   pub fn just_send(&self, request: frame_protocol::Request) -> Result<(), RequestError> {
//     self.send(frame_protocol::RequestKind::Plain, request)
//   }
// }


impl schedule::Worker<frame_protocol::Request, ()> for WorkerState {
  fn handle(&mut self, request: frame_protocol::Request, kind: frame_protocol::RequestKind) -> frame_protocol::Response {
    use frame_protocol::{Response, Request};
    use std::os::raw::c_int;

    match kind {
      frame_protocol::RequestKind::Plain => {},
      frame_protocol::RequestKind::Once(m) => {
        //TODO: Is it really true that trylock is OK, if we assume that at least one threat should be able to lock the mutex?
        if let Ok(mut guard) = m.try_lock() {
          if *guard {
            *guard = false;
          } else {
            return frame_protocol::Response::Ok(());
          }
        } else {
          return frame_protocol::Response::Ok(());
        }
      },
    }

    match request {
      Request::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, flags, param } => {
        let mut err: c_int = 0;
        let param = {
          let param_lock = param.lock().unwrap();
          param_lock.clone()
        };
        let param = &param[..];
        let param_ptr = if param.len() > 0 { param.as_ptr() } else { std::ptr::null() };
        let res = unsafe {
          ffi::vs_create_sws_context(&mut self.sws_ctx, width, height, pix_fmt, new_width, new_height, new_pix_fmt, flags as _, param_ptr, &mut err as _)
        };
        match ffi::wrap_VSResult(res, err, ()) {
          Ok(()) => Response::Ok(()),
          Err(_) => todo!(),
        }
      },
      // Request::Ping(id) => Response::Pong(id),
    }
  }
}

// impl WorkerState {
//   pub fn handle(&mut self, request: frame_protocol::Request) -> frame_protocol::Response {
//     use frame_protocol::{Response, Request};
//     match request {
//       Request::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, flags, param } => {
//         let mut err = raw::c_int::default();
//         let param = {
//           let param_lock = param.lock().unwrap();
//           param_lock.clone()
//         };
//         let param = &param[..];
//         let param_ptr = if param.len() > 0 { param.as_ptr() } else { std::ptr::null() };
//         let res = unsafe {
//           ffi::vs_create_sws_context(&mut self.sws_ctx, width, height, pix_fmt, new_width, new_height, new_pix_fmt, flags as _, param_ptr, &mut err as _)
//         };
//         match ffi::wrap_VSResult(res, err, ()) {
//             Ok(()) => Response::Ok,
//             Err(_) => todo!(),
//         }
//       },
//       // Request::Ping(id) => Response::Pong(id),
//     }
//   }
// }




