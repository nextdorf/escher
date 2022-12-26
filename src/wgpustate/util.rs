use crate::video;


pub enum EscherWGPUCallbackFn<'a> {
  // The callback function for Video Rendering in FFMPEG <-> WGPU
  RenderFrame(video::RawImageRef<'a>),
}



