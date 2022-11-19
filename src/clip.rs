use std::path;

use video;


trait VisibleT<T> {
  fn is_visibe(&self) -> bool;
  fn set_visibe(&mut self, val: bool);
  fn get(&self) -> &T;
}

struct Visible<T> {
  visibility: bool,
  var: T
}
impl<T> VisibleT<T> for Visible<T> {
  fn is_visibe(&self) -> bool { self.visibility }
  fn set_visibe(&mut self, val: bool) { self.visibility = val }
  fn get(&self) -> &T { &self.var }
}

trait Clip<Render, Timeline, Preview>
  where Render: VisibleT<egui::TextureHandle>{
  fn name(&self) -> String;
  fn set_name(&mut self, val: String);

  fn render_texture(&self) -> &Render;
  fn timeline_texture(&self) -> &Timeline;
  fn preview_texture(&self) -> &Preview;
}

struct FileClip {
  name: String,
  render_tex: Visible<egui::TextureHandle>,
  timeline_tex: Visible<egui::TextureHandle>,
  preview_tex: Visible<egui::TextureHandle>,
  vstream: video::VideoStream,
}

impl Clip<
    Visible<egui::TextureHandle>,
    Visible<egui::TextureHandle>,
    Visible<egui::TextureHandle>
  > for FileClip {
  fn name(&self) -> String { self.name.clone() }
  fn set_name(&mut self, val: String) { self.name = val }

  fn render_texture(&self) -> &Visible<egui::TextureHandle> { &self.render_tex }
  fn timeline_texture(&self) -> &Visible<egui::TextureHandle> { &self.timeline_tex }
  fn preview_texture(&self) -> &Visible<egui::TextureHandle> { &self.preview_tex }
}

impl FileClip {
  pub fn new(path: &path::Path, stream_idx: i32, render_width: i32, render_height: i32) -> Option<FileClip> {
    let vstream = Ok(video::PartialVideoStream::new()).and_then(|pvs| { pvs
        .open_format_context_from_path(path)?
        .open_codec_context(stream_idx, 0, 0)?
        .create_sws_context(render_width, render_height, video::AVPixelFormat::AV_PIX_FMT_RGB24, video::SWS_Scaling::Bilinear)?
        .create_pkt_frm()
      })
      .ok()?
      .with_current_frame();
    let name = path.file_stem()?;
    // egui::TextureHandle::
    todo!()
  }
}

