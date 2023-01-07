
use std::{ptr::{slice_from_raw_parts}, array, num::NonZeroU32};

use wgpu::{ImageDataLayout, Extent3d};

use super::{VideoStream, AVPixelFormat, AVFrame};


#[derive(Clone, Copy)]
pub struct RawImageRef<'a> {
  // _frm: &'a AVFrame,
  // _frm: *'a const AVFrame,
  planes: [&'a [u8]; 8],
  linesize: [usize; 8],
  width: usize,
  height: usize,
  pix_fmt: AVPixelFormat,
}

impl VideoStream{
  pub fn decoded_frm(&self) -> RawImageRef {
    unsafe {
      todo!()
      // RawImageRef::new(self.swsfrm.as_ref().unwrap())
    }
  }

  pub fn decoded_raw_frm(&self) -> RawImageRef {
    unsafe {
      todo!()
      // RawImageRef::new(self.frm.as_ref().unwrap())
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

    // RawImageRef { _frm: frm, planes, linesize, width, height, pix_fmt }
    RawImageRef { planes, linesize, width, height, pix_fmt }
  }

  pub fn planes(&self) -> [&[u8]; 8] { self.planes }
  pub fn linesize(&self) -> [usize; 8] { self.linesize }
  pub fn width(&self) -> usize { self.width }
  pub fn height(&self) -> usize { self.height }
  pub fn pix_fmt(&self) -> AVPixelFormat { self.pix_fmt }

  pub fn new_dummy_rgba32<'a>(data_store: &'a mut Vec<u8>, width: usize, height: usize) -> RawImageRef<'a> {
    let linesize = width*4;
    *data_store = Vec::with_capacity(linesize*height);
    for j in 0..height {
      for i in 0..width {
        let r = (i as f32)/((width-1) as f32);
        let g = (j as f32)/((height-1) as f32);
        data_store.push((r*255.).round() as _);
        data_store.push((g*255.).round() as _);
        data_store.push(0);
        data_store.push(255);
      }
    }
    RawImageRef {
      // _frm: std::ptr::null(),
      planes: [data_store.as_slice(), &[], &[], &[], &[], &[], &[], &[]],
      linesize: [linesize, 0, 0, 0, 0, 0, 0, 0],
      width: width,
      height: height,
      pix_fmt: AVPixelFormat::AV_PIX_FMT_RGBA
    }
  }

  pub fn get_image_data_layout(&self) -> ImageDataLayout {
    match self.pix_fmt {
      AVPixelFormat::AV_PIX_FMT_RGBA => ImageDataLayout {
        offset: 0,
        bytes_per_row: NonZeroU32::new(self.linesize[0] as _),
        rows_per_image: NonZeroU32::new(self.height as _),
      },
      _ => todo!(),
    }
  }  

  pub fn get_image_size(&self) -> Extent3d {
    Extent3d { width: self.width as _, height: self.height as _, depth_or_array_layers: 1 }
  }
}


