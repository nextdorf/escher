use std::num::NonZeroU32;

use wgpu::util::DeviceExt;

use crate::video;


pub enum EscherWGPUCallbackFn<'a> {
  // The callback function for Video Rendering in FFMPEG <-> WGPU
  RenderFrame(usize, Option<video::RawImageRef<'a>>),
}

/// Creates a new buffer with `COPY_SRC` as usage and pads the data to align with `COPY_BYTES_PER_ROW_ALIGNMENT`
/// if necessary. For an image `stride_len` is the image's width in bytes and `n_rows` is the image's
/// height
pub fn create_padded_copy_buffer_init(device: &wgpu::Device, label: Option<&str>, data: &[u8], stride_len: usize, n_rows: u32) -> (wgpu::Buffer, wgpu::ImageDataLayout) {
  let padded_data_store;
  let (padded_data, new_stride) = match crate::util::pad_array(
      data,
      stride_len,
      wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as _,
      true ) {
    Some((data, width)) => {
      padded_data_store = data;
      (padded_data_store.as_slice(), width)
    },
    None => (data, stride_len)
  };

  let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label,
    contents: padded_data,
    usage: wgpu::BufferUsages::COPY_SRC,
  });
  let buffer_layout = wgpu::ImageDataLayout {
    offset: 0,
    bytes_per_row: NonZeroU32::new(new_stride as _),
    rows_per_image: NonZeroU32::new(n_rows)
  };
  // eprintln!("Not sure where to place {:?}...", dbg!(texture_id));
  assert_eq!(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT, buffer_layout.bytes_per_row.unwrap().into());
  (buffer, buffer_layout)
}

