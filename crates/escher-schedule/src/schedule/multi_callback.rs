use std::{sync::mpsc::{self, Sender, Receiver}, collections::HashMap};

use super::ScheduleCallback;


pub struct ScheduleCallbackCollection<T: Copy> {
  callback_to_proxy_lookup: HashMap<u64, u64>,
  tx: Sender<(u64, u64, T)>,
  rx: Receiver<(u64, u64, T)>,
}

impl<T: Copy> ScheduleCallbackCollection<T> {
  pub fn receiver(&self) -> &Receiver<(u64, u64, T)> {
    &self.rx
  }

  pub fn sender(&self) -> &Sender<(u64, u64, T)> {
    &self.tx
  }

  pub fn lookup_map(&self) -> &HashMap<u64, u64> {
    &self.callback_to_proxy_lookup
  }

  pub fn add_callback(&mut self, callback: &mut ScheduleCallback<T>) {
    let id = callback.id();
    if !self.callback_to_proxy_lookup.contains_key(&id) {
      let proxy_id = callback.add_proxy(self.tx.clone());
      if let Some(x) = self.callback_to_proxy_lookup.insert(id, proxy_id) {
        panic!("Collision for callback {:?}", x)
      }
    }
  }

  pub fn remove_callback(&mut self, callback: &mut ScheduleCallback<T>) -> Option<u64> {
    match self.callback_to_proxy_lookup.remove(&callback.id()) {
      Some(proxy_id) => callback.remove_proxy(proxy_id).and(Some(proxy_id)),
      None => None,
    }
  }
}

impl<T: Copy> Default for ScheduleCallbackCollection<T> {
  fn default() -> Self {
    let (tx, rx) = mpsc::channel();
    Self { tx, rx, callback_to_proxy_lookup: HashMap::new() }
  }
}


