use std::{ffi::{CString, NulError}, path};

use super::{
  VSResult,
  VideoStream,
  VideoStreamErr,
  // VideoStreamResult,
  // AVFormatContext,
  // AVCodecContext,
  // AVStream,
  c_string_from_path,
  av_packet_alloc,
  av_frame_alloc,
  vs_open_format_context_from_path,
  vs_open_codec_context,
  rc::RcFrame, wrap_VSResult,
};


#[derive(Debug, Default)]
pub struct VideoStreamBuilder {
  path_cstr: Option<CString>,
  // fmt_ctx: *mut AVFormatContext,
  // codec_ctx: *mut AVCodecContext,
  // stream: *mut AVStream,
  stream_idx: u32,
  n_threads: u32,
  resolution: std::os::raw::c_int,
}

impl VideoStreamBuilder {
  pub fn set_path(mut self, path: &path::Path) -> Result<Self, NulError> {
    self.path_cstr = Some(c_string_from_path(path)?);
    Ok(self)
  }

  pub fn set_stream_idx(mut self, stream_idx: u32) -> Self {
    self.stream_idx = stream_idx;
    self
  }

  pub fn set_resolution(mut self, resolution: i32) -> Self {
    self.resolution = resolution;
    self
  }

  pub fn set_threads(mut self, n_threads: u32) -> Self {
    self.n_threads = n_threads;
    self
  }

  pub fn set_thread_to_all(self) -> Self {
    self.set_threads(0)
  }

  pub fn finish(self) -> VSResult<VideoStream> {
    let mut err = 0;
    let fmt_ctx = {
      let path_cstr = match self.path_cstr {
        Some(path_cstr) => path_cstr,
        None => return Err(VideoStreamErr::NullReference),
      };
      let mut ptr = std::ptr::null_mut();
      let res = unsafe {
        vs_open_format_context_from_path(path_cstr.as_ptr() as _, &mut ptr, &mut err)
      };
      match wrap_VSResult(res, err, ptr) {
        Ok(ptr) => ptr,
        Err(err) => return Err(err),
    }};
    let stream = unsafe {
      let streams = std::slice::from_raw_parts_mut((*fmt_ctx).streams, (*fmt_ctx).nb_streams as _);
      streams[self.stream_idx as usize]
    };
    let codec_ctx = unsafe {
      let mut ptr = std::ptr::null_mut();
      let res = vs_open_codec_context(fmt_ctx, self.stream_idx as _, self.n_threads, self.resolution, &mut ptr, &mut err);
      match wrap_VSResult(res, err, ptr) {
        Ok(ptr) => ptr,
        Err(err) => return Err(err),
    }};
    if fmt_ctx.is_null() || codec_ctx.is_null() || stream.is_null() {
      return Err(VideoStreamErr::NullReference)
    }
    let (pkt, frm) = unsafe{(
      av_packet_alloc(),
      RcFrame::wrap_raw(av_frame_alloc())
    )};
    
    let mut res = VideoStream { fmt_ctx, codec_ctx, stream, pkt, frm };
    res.decode_frames(1).and(Ok(res))
  }
}



