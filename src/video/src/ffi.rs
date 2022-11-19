#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{path, os::unix::prelude::OsStrExt, slice, };

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod video_stream;

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

impl VideoStream{
  pub fn new() -> Self {
    VideoStream {
      fmt_ctx: std::ptr::null_mut(),
      codec_ctx: std::ptr::null_mut(),
      stream: std::ptr::null_mut(),
      pkt: std::ptr::null_mut(),
      frm: std::ptr::null_mut(),
      sws_ctx: std::ptr::null_mut(),
      swsfrm: std::ptr::null_mut()
    }
  }

  fn is_valid(&self) -> bool{
    !(  self.fmt_ctx.is_null()
    ||  self.codec_ctx.is_null()
    ||  self.stream.is_null()
    ||  self.pkt.is_null()
    ||  self.frm.is_null()
    ||  self.sws_ctx.is_null()
    ||  self.swsfrm.is_null()
    )
  }

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
        seconds, flags, codec_ctx, self.pkt, self.frm, (&mut err) as _)
    }
    wrap_VSResult(res, err, ())
  }

  pub fn decode_frames(&mut self, n: u64, apply_sws_ctx: bool) -> UnitRes {
    let (sws_ctx, swsfrm) = if apply_sws_ctx {
      (self.sws_ctx, self.swsfrm)
    } else {
      (std::ptr::null_mut(), std::ptr::null_mut())
    };
    let mut err: i32 = 0;
    let res;
    unsafe{
      res = vs_decode_frames(self.fmt_ctx, self.codec_ctx, self.stream, self.pkt, self.frm,
        sws_ctx, swsfrm, n, (&mut err) as _);
    }
    wrap_VSResult(res, err, ())
  }

}

impl Drop for VideoStream{
  fn drop(&mut self) {
    unsafe {
      vs_free(self)
    }
  }
}

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

fn wrap_VSResult<T>(res: VideoStreamResult, err: i32, x: T) -> Result<T, VideoStreamErr> {
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


pub struct PartialVideoStream {
  val: VideoStream
}

impl TryFrom<PartialVideoStream> for VideoStream {
  type Error = VideoStreamErr;

  fn try_from(pvs: PartialVideoStream) -> Result<Self, Self::Error> {
    let vs = pvs.val;
    if vs.is_valid() {
      Ok(vs)
    } else {
      Err(VideoStreamErr::NullReference)
    }
  }
}


type VSResult<T> = Result<T, VideoStreamErr>;
type UnitRes = VSResult<()>;
// type VideoStreamRes = VSResult<VideoStream>;

impl PartialVideoStream {
  pub fn new() -> Self{
    PartialVideoStream { val: VideoStream::new() }
  }

  pub fn open_format_context_from_path(mut self, path: &path::Path) -> VSResult<Self> {
    let fmt_ctx_ptr = (&mut self.val.fmt_ctx) as _;
    let path_ptr = path.as_os_str().as_bytes().as_ptr() as _;
    let mut err: i32 = 0;
    let res;
    unsafe{
      res = vs_open_format_context_from_path(path_ptr, fmt_ctx_ptr, (&mut err) as _);
    }
    wrap_VSResult(res, err, self)
  }

  pub fn open_codec_context(mut self, stream_idx: i32, nThreads: u32, resolution: i32) -> VSResult<Self>{
    let fmt_ctx = self.val.fmt_ctx;
    let codec_ctx_ptr = (&mut self.val.codec_ctx) as _;
    let mut err: i32 = 0;
    let res;
    unsafe{
      let streams = slice::from_raw_parts_mut((*self.val.fmt_ctx).streams, (*self.val.fmt_ctx).nb_streams as _);
      self.val.stream = streams[stream_idx as usize];
  
      res = vs_open_codec_context(fmt_ctx, stream_idx, nThreads, resolution, codec_ctx_ptr, (&mut err) as _);
    }
    wrap_VSResult(res, err, self)
  }

  pub fn create_sws_context(mut self, new_width: i32, new_height: i32, new_pix_fmt: AVPixelFormat, scaling: SWS_Scaling) -> VSResult<Self>{
    let codec_ctx = self.val.codec_ctx;
    let sws_ctx_ptr = (&mut self.val.sws_ctx) as _;
    let (flags, param) = match scaling {
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
    };
    let param = &param[..];
    let param_ptr = if param.len() > 0 { param.as_ptr() } else {std::ptr::null_mut()};
    let mut err: i32 = 0;
    let res;
    unsafe{
      res = vs_create_sws_context(codec_ctx, sws_ctx_ptr, new_width, new_height, new_pix_fmt, flags as _, param_ptr, (&mut err) as _);
    }
    wrap_VSResult(res, err, self)
  }

  pub fn create_pkt_frm(mut self) -> VSResult<Self> {
    let pkt_ptr = (&mut self.val.pkt) as _;
    let frm_ptr = (&mut self.val.frm) as _;
    let swsfrm_ptr = (&mut self.val.swsfrm) as _;
    let res;
    unsafe{
      res = vs_create_pkt_frm(pkt_ptr, frm_ptr, swsfrm_ptr);
    }
    wrap_VSResult(res, -1, self)
  }

  pub fn with_current_frame(mut self) -> VideoStream{
    self.val.decode_frames(0, true).expect("PartialVideoStream wasn't fully initialized before casting it to VideoStream");
    self.val
  }

  pub fn fmap<T>(self, f: &dyn Fn(&VideoStream) -> VSResult<T>) -> VSResult<T> {
    f(&self.val)
  }

  pub fn fmapMut(mut self, f: impl Fn(&mut VideoStream) -> UnitRes) -> VSResult<Self> {
    match f(&mut self.val) {
      Ok(()) => Ok(PartialVideoStream { val: self.val }),
      Err(e) => Err(e)
    }
  }
}
