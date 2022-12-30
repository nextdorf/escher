use std::{collections::{HashMap, self}, sync::{Arc, Mutex}, path::Path, fmt::Debug};

use egui_winit::egui::{self, Widget};
use epaint::vec2;

pub trait Asset: Debug {
  fn get_name(&self) -> String;
  fn get_texture_handle(&self) -> egui::TextureHandle;
  fn as_widget<'a: 'b, 'b>(&'a self) -> Box<dyn FnOnce(&mut egui::Ui) -> egui::Response + 'b> {
    let name = self.get_name();
    let tex_handle = self.get_texture_handle();
    Box::new(|ui| asset_ui(name, tex_handle, ui))
  }
}

#[derive(Default)]
pub struct AssetManager {
  next_id: usize,
  assets: HashMap<usize, Arc<Mutex<dyn Asset>>>
}

impl AssetManager {
  pub fn add<T: Asset + 'static>(&mut self, asset: T) -> Result<(), (usize, Arc<Mutex<dyn Asset>>)> {
    let id = self.next_id;
    self.next_id += 1;
    let asset = Arc::new(Mutex::new(asset));
    match self.assets.insert(id, asset) {
      Some(colliding) => Err((id, colliding)),
      None => Ok(()),
    }
  }

  pub fn iter(&self) -> collections::hash_map::Values<usize, Arc<Mutex<dyn Asset>>> {
    self.assets.values()
  }
}


pub struct DummyAsset {
  pub name: String,
  pub tex_handle: egui::TextureHandle,
}

impl DummyAsset {
  pub fn load(path: &Path, ctx: &egui::Context) -> Self {
    let path = Box::new(path.clone());
    let name = match path.file_stem() {
      Some(stem) => match stem.to_str() {
        Some(s) => s.to_string(),
        None => "Datei".to_string()
      },
      None => "Datei".to_string(),
    };
    // ctx.load_texture(name, image, options)
    todo!()
  }

  pub fn load_default(ctx: &egui::Context) -> Self{
    let tex_handle = ctx.load_texture(
      "sample_texture",
      egui::ColorImage::example(),
      egui::TextureOptions::default()
    );
    let name = "Sample Sample Sample Sample Sample Sample Sample Sample".to_string();
    DummyAsset {
      name,
      tex_handle,
    }
  }


}

impl Asset for DummyAsset {
  fn get_name(&self) -> String {
    self.name.clone()
  }

  fn get_texture_handle(&self) -> egui::TextureHandle {
    self.tex_handle.clone()
  }

}

pub fn asset_ui(name: String, tex_handle: egui::TextureHandle, ui: &mut egui::Ui) -> egui::Response {
  let img_size = 64.;
  let label_size = 20.;
  // egui::PaintCallback
  let (rect, resp) = ui.allocate_exact_size(vec2(img_size, img_size + label_size), egui::Sense::hover());

  let img_rect = egui::Rect { max: egui::pos2(rect.right(), rect.top() + img_size), ..rect };
  let text_rect = egui::Rect { min: egui::pos2(rect.left(), rect.top() + img_size), ..rect };

  // See egui::Image::ui impl for Widget
  egui::Image::new(tex_handle.id(), vec2(img_size, img_size))
    .paint_at(ui, img_rect);
  // egui::Label::new(self.get_name()).ui(ui);

  // See egui::Label::ui impl for Widget
  let style = ui.style();
  let font_id = style.text_styles.get(&egui::TextStyle::Body).unwrap();
  let text_color = style.visuals.text_color();
  let job = epaint::text::LayoutJob::simple_singleline(name, font_id.clone(), text_color);
  let galley = ui.fonts().layout_job(job);

  ui.painter_at(text_rect)
    .add(epaint::TextShape {
      pos: text_rect.left_top(),
      galley,
      underline: epaint::Stroke::NONE,
      override_text_color: None,
      angle: 0.,
  });

  // ui.painter().add(shape)
  
  // ui.label(text)
  resp
}


impl Debug for DummyAsset {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DummyAsset").field("name", &self.name).field("tex_handle", &self.tex_handle.id()).finish()
  }
}


