use std::{collections::{HashMap, self}, sync::{Arc, Mutex}, path::Path, fmt::Debug};

use egui_winit::egui;

pub trait Asset: Debug {
  fn get_name(&self) -> String;
  fn get_texture_handle(&self) -> egui::TextureHandle;
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
    let name = "Sample".to_string();
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

impl Debug for DummyAsset {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DummyAsset").field("name", &self.name).field("tex_handle", &self.tex_handle.id()).finish()
  }
}


