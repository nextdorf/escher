use std::{ptr::{slice_from_raw_parts}, array};

use super::{VideoStream, AVPixelFormat, AVFrame};


pub struct RawImageRef<'a> {
  _frm: &'a AVFrame,
  planes: [&'a [u8]; 8],
  linesize: [usize; 8],
  width: usize,
  height: usize,
  pix_fmt: AVPixelFormat,
}

impl VideoStream{
  pub fn decoded_frm(&self) -> RawImageRef {
    unsafe {
      RawImageRef::new(self.swsfrm.as_ref().unwrap())
    }
  }

  pub fn decoded_raw_frm(&self) -> RawImageRef {
    unsafe {
      RawImageRef::new(self.frm.as_ref().unwrap())
    }
  }

}


impl RawImageRef<'_> {
  pub fn new(frm: &AVFrame) -> RawImageRef {
    let linesize = frm.linesize.map(|i| i as _);
    let width = frm.width as _;
    let height = frm.height as _;
    let pix_fmt = unsafe{ std::mem::transmute(frm.format) };
    let planes = array::from_fn(|i| {
      let datalen = linesize[i]*height;
      if datalen > 0 {
        let plane = slice_from_raw_parts(
          frm.data[i],
          datalen);
        unsafe{ plane.as_ref().unwrap() }
      } else {
        &[]
      }
    });

    RawImageRef { _frm: frm, planes, linesize, width, height, pix_fmt }
  }

  pub fn planes(&self) -> [&[u8]; 8] { self.planes }
  pub fn linesize(&self) -> [usize; 8] { self.linesize }
  pub fn width(&self) -> usize { self.width }
  pub fn height(&self) -> usize { self.height }
  pub fn pix_fmt(&self) -> AVPixelFormat { self.pix_fmt }
}


