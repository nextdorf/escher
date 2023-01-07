use std::ops::{Deref, DerefMut};

use super::{AVFrame, av_frame_ref, av_buffer_get_ref_count, av_frame_unref, av_frame_clone};


pub struct RcFrame {
  frm: *mut AVFrame,
}

impl RcFrame {
  pub fn wrap_raw(frm: *mut AVFrame) -> Self {
    Self { frm }
  }

  pub fn wrap_null() -> Self {
    Self { frm: std::ptr::null_mut() }
  }

  pub fn ref_count(&self) -> [i32; 8] {
    self.buf.map(|b| unsafe{av_buffer_get_ref_count(b)})
  }

  pub fn clone_from_raw(&mut self, src: *const AVFrame) -> super::UnitRes {
    let err = unsafe {
      av_frame_unref(self.frm);
      av_frame_ref(self.frm, src)
    };
    if err == 0 {
      Ok(())
    } else {
      Err(super::VideoStreamErr::FFMPEGErr { err })
    }
  }

  pub fn clone_new_from_raw(src: *const AVFrame) -> Self {
    Self { frm: unsafe{av_frame_clone(src)} }
  }

  pub fn leak(&self) -> *const AVFrame {
    self.frm
  }
  pub fn leak_mut(&mut self) -> *mut AVFrame {
    self.frm
  }

}


impl Clone for RcFrame {
  fn clone(&self) -> Self {
    Self::clone_new_from_raw(self.frm)
  }

fn clone_from(&mut self, source: &Self) {
    self.clone_from_raw(source.frm).unwrap()
  }
}

impl Drop for RcFrame {
  fn drop(&mut self) {
    unsafe {av_frame_unref(self.frm)}
  }
}

impl Deref for RcFrame {
  type Target = AVFrame;

  fn deref(&self) -> &Self::Target {
    unsafe {&*self.frm}
  }
}

impl DerefMut for RcFrame {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe{&mut *self.frm}
  }
}

unsafe impl Send for RcFrame {}
unsafe impl Sync for RcFrame {}

