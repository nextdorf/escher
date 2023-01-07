#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{path, ffi::CString};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
// use VideoFrame as VideoFrameC;

pub mod video_stream;
pub mod rc;
mod video_stream_builder;
pub use video_stream_builder::VideoStreamBuilder;

// #[macro_use]
use bitflags::bitflags;

bitflags! {
  pub struct Seek: u32 {
    const Backward = AVSEEK_FLAG_BACKWARD;
    const Byte = AVSEEK_FLAG_BYTE;
    const Any = AVSEEK_FLAG_ANY;
    const Frame = AVSEEK_FLAG_FRAME;
    const NoFastMode = 1 << 30;
    const NoPreciseMode = 1 << 31;
    const FlagsOnly = !(Self::NoFastMode.bits | Self::NoPreciseMode.bits);
  }
}


pub struct VideoStream {
  fmt_ctx: *mut AVFormatContext,
  codec_ctx: *mut AVCodecContext,
  stream: *mut AVStream,
  pkt: *mut AVPacket,
  frm: rc::RcFrame,
}

pub struct VideoFrameContext {
  pub frm_src: rc::RcFrame,
  pub(crate) sws_ctx: *mut SwsContext,
  pub(crate) sws_frm: rc::RcFrame,
}


impl VideoStream{
  pub fn seek(&mut self, seconds: f64, flags: Seek) -> UnitRes {
    let codec_ctx = if (flags & Seek::NoPreciseMode).is_empty() {
        self.codec_ctx
      } else {
        std::ptr::null_mut()
      };
    let flags = if (flags & Seek::NoFastMode).is_empty() {
        (flags & Seek::FlagsOnly).bits as _
      } else {
        -1
      };
    let mut err: i32 = 0;
    let res;
    unsafe{
      res = vs_seek_at(self.fmt_ctx, self.stream,
        seconds, flags, codec_ctx, self.pkt, self.frm.leak_mut(), (&mut err) as _)
    }
    wrap_VSResult(res, err, ())
  }

  pub fn decode_frames(&mut self, n: u64) -> UnitRes {
    let mut err = 0;
    let res = unsafe{
      let (sws_ctx, swsfrm) = (std::ptr::null_mut(), std::ptr::null_mut());
      vs_decode_frames(self.fmt_ctx, self.codec_ctx, self.stream, self.pkt, self.frm.leak_mut(), sws_ctx, swsfrm, n, &mut err)
    };
    wrap_VSResult(res, err, ())
  }

  pub fn get_frm(&self) -> rc::RcFrame {
    self.frm.clone()
  }
}

impl Drop for VideoStream{
  fn drop(&mut self) {
    unsafe {
      let mut frm = self.frm.leak_mut();
      if !frm.is_null() {
        av_frame_unref(frm);
        av_frame_free(&mut frm);
      }
      if !self.pkt.is_null() {
        av_packet_unref(self.pkt);
        av_packet_free(&mut self.pkt);
      }
      if !self.codec_ctx.is_null() {
        avcodec_close(self.codec_ctx);
      }
      if !self.fmt_ctx.is_null() {
        avformat_close_input(&mut self.fmt_ctx);
      }
    }
  }
}


impl VideoFrameContext {
  pub fn new(src: rc::RcFrame) -> Self {
    let sws_frm = rc::RcFrame::wrap_raw(unsafe{av_frame_alloc()});
    Self { frm_src: src, sws_ctx: std::ptr::null_mut(), sws_frm }
  }
  pub fn new_init(src: rc::RcFrame, new_width: i32, new_height: i32, new_pix_fmt: AVPixelFormat, width: i32, height: i32, pix_fmt: AVPixelFormat, scaling: SWS_Scaling) -> VSResult<Self> {
    let mut res = Self::new(src);
    res.replace_sws_ctx(new_width, new_height, new_pix_fmt, width, height, pix_fmt, scaling)
      .and(Ok(res))
  }

  pub fn replace_sws_ctx(&mut self, new_width: i32, new_height: i32, new_pix_fmt: AVPixelFormat, width: i32, height: i32, pix_fmt: AVPixelFormat, scaling: SWS_Scaling) -> UnitRes {
    let mut err = 0;
    let mut sws_ctx = std::ptr::null_mut();
    let res = unsafe {
      let (flags, param) = scaling.into();
      let param = if param.len()>0 {
        param[..].as_ptr()
      } else {
        std::ptr::null()
      };
      vs_create_sws_context(&mut sws_ctx, width, height, pix_fmt, new_width, new_height, new_pix_fmt, flags as _, param, &mut err)
    };
    match wrap_VSResult(res, err, sws_ctx) {
      Ok(sws_ctx) => {
        // let sws_frm = rc::RcFrame::wrap_raw(unsafe{av_frame_alloc()});
        // Ok(Self { frm_src: src, sws_ctx, sws_frm })
        self.sws_ctx = sws_ctx;
        Ok(())
      },
      Err(err) => Err(err)
    }
  }

  pub fn decode(&mut self) -> UnitRes {
    let mut err = 0;
    let res = unsafe {
      vf_decode_sws_frame(self.frm_src.leak_mut(), self.sws_ctx, self.sws_frm.leak_mut(), &mut err)
    };
    wrap_VSResult(res, err, ())
  }
}

impl Drop for VideoFrameContext {
  fn drop(&mut self) {
    unsafe {
      // drop(self.frm_src);
      if !self.sws_ctx.is_null() {
        sws_freeContext(self.sws_ctx);
      }
      let mut sws_frm = std::mem::replace(&mut self.sws_frm, rc::RcFrame::wrap_null());
      av_frame_free(&mut sws_frm.leak_mut())
      // if !self.sws_frm.is_null() {
      //   av_frame_unref(self.sws_frm);
      //   av_frame_free(&mut self.sws_frm);
      // }
    }
  }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SWS_Scaling {
  FastBilinear,
  Bilinear,
  Bicubic {p1: f64, p2: f64},
  X,
  Point,
  Area,
  Bicublin,
  Gauss {exponent: f64},
  Sinc,
  Lanczos {window_width: f64},
  Spline,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum VideoStreamErr{
  FFMPEGErr {err: i32},
  TimeStampOutOfBounds,
  EOF,
  IO,
  EncoderTrysToDecode,
  DecoderTrysToEncode,
  IndexOutOfBounds,
  StreamNotFound,
  DecoderNotFound,
  NullReference
}

pub fn wrap_VSResult<T>(res: VideoStreamResult, err: i32, x: T) -> Result<T, VideoStreamErr> {
  match res {
    VideoStreamResult::vs_ffmpeg_errorcode => Err(VideoStreamErr::FFMPEGErr { err }),
    VideoStreamResult::vs_success => Ok(x),
    VideoStreamResult::vs_timestamp_out_of_bounds => Err(VideoStreamErr::TimeStampOutOfBounds),
    VideoStreamResult::vs_eof => Err(VideoStreamErr::EOF),
    VideoStreamResult::vs_io => Err(VideoStreamErr::IO),
    VideoStreamResult::vs_encoder_trys_to_decode => Err(VideoStreamErr::EncoderTrysToDecode),
    VideoStreamResult::vs_decoder_trys_to_encode => Err(VideoStreamErr::DecoderNotFound),
    VideoStreamResult::vs_index_out_of_bounds => Err(VideoStreamErr::IndexOutOfBounds),
    VideoStreamResult::vs_stream_not_found => Err(VideoStreamErr::StreamNotFound),
    VideoStreamResult::vs_decoder_not_found => Err(VideoStreamErr::DecoderNotFound),
    VideoStreamResult::vs_null_reference => Err(VideoStreamErr::NullReference),
  }
}


type VSResult<T> = Result<T, VideoStreamErr>;
type UnitRes = VSResult<()>;
// type VideoStreamRes = VSResult<VideoStream>;


#[cfg(target_family = "unix")]
fn c_string_from_path(path: &path::Path) -> Result<CString, std::ffi::NulError> {
  use std::os::unix::prelude::OsStrExt;
  CString::new(path.as_os_str().as_bytes())
}
#[cfg(not(target_family = "unix"))]
fn c_string_from_path(path: &path::Path) -> Result<CString, std::ffi::NulError> {
  let s = path.to_string_lossy().to_string();
  CString::new(s.as_bytes())
}


impl From<SWS_Scaling> for (std::os::raw::c_uint, Vec<f64>) {
    fn from(value: SWS_Scaling) -> Self {
      match value {
        SWS_Scaling::FastBilinear => (SWS_FAST_BILINEAR, vec![]),
        SWS_Scaling::Bilinear => (SWS_BILINEAR, vec![]),
        SWS_Scaling::Bicubic { p1, p2 } => (SWS_BICUBIC, vec![p1, p2]),
        SWS_Scaling::X => (SWS_X, vec![]),
        SWS_Scaling::Point => (SWS_POINT, vec![]),
        SWS_Scaling::Area => (SWS_AREA, vec![]),
        SWS_Scaling::Bicublin => (SWS_BICUBLIN, vec![]),
        SWS_Scaling::Gauss { exponent } => (SWS_GAUSS, vec![exponent]),
        SWS_Scaling::Sinc => (SWS_SINC, vec![]),
        SWS_Scaling::Lanczos { window_width } => (SWS_LANCZOS, vec![window_width]),
        SWS_Scaling::Spline => (SWS_SPLINE, vec![]),
    }
  }
}

