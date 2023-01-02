use crate::ffi::{AVFrame, self};

use std::{sync::{Arc, Mutex, mpsc, self}, os::raw, thread::JoinHandle, rc};


pub mod frame_protocol {
  use std::sync::{Arc, Mutex};

  use crate::ffi;

  pub enum Request {
    SetSwsContext {new_width: i32, new_height: i32, new_pix_fmt: ffi::AVPixelFormat, width: i32, height: i32, pix_fmt: ffi::AVPixelFormat, flags: u32, param: Arc<Mutex<Vec<f64>>>},
    // Ping(usize),
  }
  pub enum Response {
    Ok,
    // Pong(usize),
  }
}

pub struct FrameBuffer {
  frames: Vec<Arc<Mutex<Option<AVFrame>>>>,
  workers: Vec<Option<(mpsc::Sender<frame_protocol::Request>, mpsc::Receiver<frame_protocol::Response>, bool, JoinHandle<()>)>>,
  weak_ref: sync::Weak<Self>,
  next_ping_id: usize, //ping pong was a nonsene idea
}

struct FrameBufferWorker {
  thread_idx: usize,
  rx: mpsc::Receiver<frame_protocol::Request>,
  tx: mpsc::Sender<frame_protocol::Response>,
  state: WorkerState,
}
struct WorkerState {
  sws_ctx: *mut ffi::SwsContext,
}


impl FrameBuffer {
  pub fn new(&self, n_threads: usize, new_width: i32, new_height: i32, new_pix_fmt: ffi::AVPixelFormat, width: i32, height: i32, pix_fmt: ffi::AVPixelFormat, scaling: ffi::SWS_Scaling) -> Arc<FrameBuffer>
  {
    use frame_protocol::Request;
    let (flags, param) = scaling.into();
    let param = Arc::new(Mutex::new(param));
    let mut workers = Vec::with_capacity(n_threads);
    for i in 0..n_threads {
      let (t_resp, r_resp) = mpsc::channel();
      let (t_req, thread) = FrameBufferWorker::new(i, t_resp);
      workers.push(Some((t_req, r_resp, false, thread)));
    }
    for w in workers.iter() {
      let tx = &w.as_ref().unwrap().0;
      let param = param.clone();
      tx.send(Request::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, flags, param }).unwrap();
    }
    
    Arc::new_cyclic(|weak_ref| {
      Self { frames: Vec::default(), workers, weak_ref: weak_ref.clone(), next_ping_id: 0 }
    })
  }

  pub fn request(&self, request: frame_protocol::Request){
    todo!()
  }

  pub fn handle_respones(&mut self) {
    for w in self.workers.iter_mut() {
      if let Some((_, rx, is_ready, thread)) = w {
        if thread.is_finished() {
          *w = None;
          continue;
        } else if *is_ready {
          continue;
        } else {
          match rx.try_recv() {
            Ok(first_val) => {
              let iter = std::iter::once(first_val).chain(rx.try_iter());
              todo!()
            },
            Err(mpsc::TryRecvError::Disconnected) => {*w = None},
            Err(mpsc::TryRecvError::Empty) => {*is_ready = true},
          }
        }
      }
    }
  }
}

impl FrameBufferWorker {
  pub fn new(thread_idx: usize, t_resp: mpsc::Sender<frame_protocol::Response>) -> (mpsc::Sender<frame_protocol::Request>, JoinHandle<()>) {
    use frame_protocol::Response;
    let (tx, rx) = mpsc::channel();
    let t = std::thread::spawn(move || {
      let (t_req, r_req) = mpsc::channel::<frame_protocol::Request>();
      tx.send(t_req).unwrap();
      t_resp.send(Response::Ok).unwrap();
      let worker_state = WorkerState { sws_ctx: std::ptr::null_mut() };
      let mut worker = Self { thread_idx, rx: r_req, tx: t_resp, state: worker_state};

      loop {
        match worker.rx.recv() {
          Ok(req) => match worker.tx.send(worker.state.handle(req)) {
            Ok(()) => {},
            Err(e) => eprintln!("Response Error: {:?}", e)
          },
          Err(e) => eprintln!("Request Error: {:?}", e)
        }
      }
    });
    (rx.recv().unwrap(), t)
  }
}

impl WorkerState {
  pub fn handle(&mut self, request: frame_protocol::Request) -> frame_protocol::Response {
    use frame_protocol::{Response, Request};
    match request {
      Request::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, flags, param } => {
        let mut err = raw::c_int::default();
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
            Ok(()) => Response::Ok,
            Err(_) => todo!(),
        }
      },
      // Request::Ping(id) => Response::Pong(id),
    }
  }
}


