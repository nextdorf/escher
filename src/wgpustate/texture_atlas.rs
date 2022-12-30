use std::collections::HashMap;
use wgpu::{Texture, BindGroup};


#[derive(Default)]
pub struct TextureAtlas {
  next_id: usize,
  inner: HashMap<usize, (Texture, BindGroup)>,
}

impl TextureAtlas {
  pub fn insert(&mut self, texture: Texture, bindgroup: BindGroup) -> usize {
    let id = self.next_id;
    self.next_id += 1;
    if let Some(_) = self.inner.insert(id, (texture, bindgroup)) {
      panic!("Exceeded limit of textures to store (2^[pointer size])")
    }
    id
  }

  pub fn remove(&mut self, id: &usize) -> Option<(Texture, BindGroup)> {
    self.inner.remove(id)
  }

  pub fn get(&self, id: &usize) -> Option<&(Texture, BindGroup)> {
    self.inner.get(id)
  }

  pub fn get_mut(&mut self, id: &usize) -> Option<(Texture, BindGroup)> {
    self.get_mut(id)
  }

}

