use crate::ffi::{rc::RcFrame, self, VideoStream, VideoFrameContext};

use std::{sync::{Arc, Mutex, mpsc}};

use escher_schedule as schedule;

pub mod frame_protocol {
  use std::fmt::Debug;
  pub use escher_schedule::RequestKind; 

  use crate::ffi;

  #[derive(Debug, Copy, Clone)]
  pub enum Request {
    SetSwsContext {new_width: i32, new_height: i32, new_pix_fmt: ffi::AVPixelFormat, width: i32, height: i32, pix_fmt: ffi::AVPixelFormat, scaling: ffi::SWS_Scaling},
    // Ping(usize),
    RenderFrame {render_idx: usize, },
  }
  // type RequestKind = escher_schedule::RequestKind;
  pub type Response = escher_schedule::Response<()>;

  // impl Clone for Request {
  //   fn clone(&self) -> Self {
  //     match self {
  //       Self::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, flags, param } => Self::SetSwsContext { new_width: new_width.clone(), new_height: new_height.clone(), new_pix_fmt: new_pix_fmt.clone(), width: width.clone(), height: height.clone(), pix_fmt: pix_fmt.clone(), flags: flags.clone(), param: param.clone() },
  //       _ => todo!(),
  //     }
  //   }
  // }

  // impl Debug for Request {
  //   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
  //     match self {
  //       Self::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, .. } => 
  //         f.debug_struct("SetSwsContext").field("new_width", new_width).field("new_height", new_height)
  //         .field("new_pix_fmt", new_pix_fmt).field("width", width).field("height", height)
  //         .field("pix_fmt", pix_fmt)
  //         // .field("flags", flags).field("param", param)
  //         .finish(),
  //       _ => todo!(),
  //     }
  //   }
  // }
}

pub use schedule::BroadcastKind;
pub type RequestError = schedule::RequestError<frame_protocol::Request>;

pub struct FrameBuffer {
  pub frames: Vec<Arc<Mutex<Option<RcFrame>>>>,
  scheduler: schedule::Scheduler<frame_protocol::Request, ()>
}


struct WorkerState {
  // sws_ctx: *mut ffi::SwsContext,
  vframe_ctx: VideoFrameContext,
}

impl FrameBuffer {
  pub fn new(src: &RcFrame, num_workers: usize, new_width: i32, new_height: i32, new_pix_fmt: ffi::AVPixelFormat, width: i32, height: i32, pix_fmt: ffi::AVPixelFormat, scaling: ffi::SWS_Scaling) -> Arc<Self>
  {
    use frame_protocol::Request;
    let src: &'static RcFrame = unsafe{std::mem::transmute(src)};
    let scheduler = schedule::Scheduler::new(num_workers, |_| WorkerState {
      // vframe_ctx: VideoFrameContext::new(RcFrame::wrap_null())
      vframe_ctx: VideoFrameContext::new(src.clone())
    });
    // let (flags, param) = scaling.into();
    for i in 0..num_workers {
      // let param = Arc::new(Mutex::new(param.clone()));
      scheduler.request(
        Request::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, scaling },
        BroadcastKind::Specific(i)
      ).unwrap();
    }
    let res = Arc::new(Self { frames: Vec::default(), scheduler });
    res
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
      Request::SetSwsContext { new_width, new_height, new_pix_fmt, width, height, pix_fmt, scaling } => {
        // let mut err: c_int = 0;
        // let param = {
        //   let param_lock = param.lock().unwrap();
        //   param_lock.clone()
        // };
        // let param = &param[..];
        // let param_ptr = if param.len() > 0 { param.as_ptr() } else { std::ptr::null() };
        // let res = unsafe {
        //   ffi::vs_create_sws_context(&mut self.sws_ctx, width, height, pix_fmt, new_width, new_height, new_pix_fmt, flags as _, param_ptr, &mut err as _)
        // };
        // let res = self.vframe_ctx.replace_sws_ctx(new_width, new_height, new_pix_fmt, width, height, pix_fmt, scaling);
        match self.vframe_ctx.replace_sws_ctx(new_width, new_height, new_pix_fmt, width, height, pix_fmt, scaling) {
          Ok(()) => Response::Ok(()),
          Err(_) => todo!(),
        }
      },
      // Request::Ping(id) => Response::Pong(id),
      _ => todo!()
    }
  }
}




