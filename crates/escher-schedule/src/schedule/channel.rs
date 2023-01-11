use std::{sync::{Mutex, mpsc, Arc, RwLock}, collections::HashMap};


pub struct CallbackMessage<T> {
  id: u64,
  callback_id: u64,
  message: T,
}

static NEXT_ID: Mutex<u64> = Mutex::new(0);

fn new_id() -> u64 {
  let mut guard = NEXT_ID.lock().unwrap();
  let id = *guard;
  *guard += 1;
  id
}


#[derive(Clone)]
pub struct CallbackSender<T> {
  senders: Vec<(u64, mpsc::Sender<CallbackMessage<T>>)>,
  id: u64,
  next_message_id: u64,
}


#[derive(Default)]
pub struct CallbackReceiver<T> {
  channel: Option<(mpsc::Sender<CallbackMessage<T>>, mpsc::Receiver<CallbackMessage<T>>)>,
  callback_to_proxy: HashMap<u64, u64>,
}

impl<T> CallbackSender<T> {
  pub fn add_callback(&mut self, callback: mpsc::Sender<CallbackMessage<T>>) -> u64 {
    let id = new_id();
    self.senders.push((id, callback));
    id
  }

  pub fn remove_callback(&mut self, id: u64) -> Option<mpsc::Sender<CallbackMessage<T>>> {
    let mut i = 0;
    for (i_id, _) in self.senders.iter() {
      if *i_id == id {
        return Some(self.senders.swap_remove(i).1);
      }
      i += 1;
    }
    None
  }

  pub fn iter_callback_ids(&self) -> impl Iterator<Item=u64> + '_ {
    self.senders.iter().map(|(id, _)| *id)
  } 

  pub fn id(&self) -> u64 {
    self.id
  }

  pub fn next_message_id(&self) -> u64 {
    self.next_message_id
  }

} 


impl<T> CallbackReceiver<T> {
  pub fn receiver(&self) -> Option<&mpsc::Receiver<CallbackMessage<T>>> {
    if let Some((_, rx)) = &self.channel {
      Some(rx)
    } else {
      None
    }
  }

  pub fn add_proxy(&mut self, callback: &mut CallbackSender<T>) -> Option<u64> {
    let callback_id = callback.id;
    if !self.callback_to_proxy.contains_key(&callback_id) {
      let (tx, _) = self.channel.get_or_insert_with(mpsc::channel);
      let proxy_id = callback.add_callback(tx.clone());
      if let Some(_) = self.callback_to_proxy.insert(callback_id, proxy_id) {
        panic!("Id collision for {}", callback_id)
      }
      Some(proxy_id)
    } else {
      None
    }
  }
  
  pub fn add_proxy_remotely(&mut self, callback_id: u64, remote: mpsc::SyncSender<mpsc::Sender<CallbackMessage<T>>>, remote_response: mpsc::Receiver<u64>, )
    -> Option<impl FnMut() -> Option<u64> + '_>
  {
    if !self.callback_to_proxy.contains_key(&callback_id) {
      let (tx, _) = self.channel.get_or_insert_with(mpsc::channel);
      if remote.send(tx.clone()).is_err() {
        return None;
      }
      Some(move || {
        let proxy_id = match remote_response.recv() {
          Ok(id) => id,
          Err(_) => return None
        };
        if let Some(_) = self.callback_to_proxy.insert(callback_id, proxy_id) {
          panic!("Id collision for {}", callback_id)
        }
        Some(proxy_id)
      })
    } else {
      None
    }
  }
  
  pub fn remove_proxy(&mut self, callback: &mut CallbackSender<T>) -> Option<u64> {
    if let Some(proxy_id) = self.callback_to_proxy.remove(&callback.id) {
      callback.remove_callback(proxy_id).and(Some(proxy_id))
    } else {
      None
    }
  }
}


impl<T: Clone> CallbackSender<T> {
  pub fn send(&mut self, x: &T) {
    // let msg_id = self.next_message_id;
    let mut i = 0;
    while let Some((_, sender)) = self.senders.get(i) {
      match sender.send(CallbackMessage { id: self.next_message_id, callback_id: self.id, message: x.clone() }) {
        Ok(()) => i += 1,
        Err(_) => { self.senders.swap_remove(i); }
      }
    }
    if i > 0 {
      self.next_message_id += 1;
    }
  }
} 


impl<T> Default for CallbackSender<T> {
  fn default() -> Self {
    Self {
      senders: Vec::new(),
      id: new_id(),
      next_message_id: 0,
    }
  }
}


