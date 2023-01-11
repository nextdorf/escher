use std::sync::{mpsc::{self, Sender, Receiver}, Mutex};


static NEXT_ID: Mutex<u64> = Mutex::new(0);

fn new_id() -> u64 {
  let mut guard = NEXT_ID.lock().unwrap();
  let res = *guard;
  *guard += 1;
  res
}

pub struct ScheduleCallback<T> {
  tx: Sender<(u64, T)>,
  rx: Receiver<(u64, T)>,
  proxy: Vec<(u64, Sender<(u64, u64, T)>)>,
  callback_id: u64,
  next_id: u64,
}

impl<T:Copy> ScheduleCallback<T> {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel();
    Self { tx, rx, proxy: Vec::new(), callback_id: new_id(), next_id: 0 }
  }

  pub fn id(&self) -> u64 {
    self.callback_id
  }

  pub fn next_message_id(&self) -> u64 {
    self.next_id
  }


  pub fn add_proxy(&mut self, tx: Sender<(u64, u64, T)>) -> u64 {
    let id = self.next_id;
    self.next_id += 1;
    self.proxy.push((id, tx));
    id
  }

  pub fn remove_proxy(&mut self, proxy_id: u64) -> Option<Sender<(u64, u64, T)>> {
    let mut i = 0;
    for (id, _) in self.proxy.iter() {
      if *id == proxy_id {
        return Some(self.proxy.swap_remove(i).1);
      }
      i += 1;
    }
    None
  }

  pub fn proxy_ids<I>(&self) -> I where I: FromIterator<u64> {
    self.proxy.iter().map(|(id, _)| *id).collect()
  }


  pub fn send(&mut self, x: T) -> Result<u64, mpsc::SendError<T>> {
    let id = self.next_id;
    self.next_id += 1;
    match self.tx.send((id, x)) {
      Ok(()) => Ok(id),
      Err(mpsc::SendError((_, err))) => Err(mpsc::SendError(err)),
    }
  }


  pub fn recv(&self) -> Result<(u64, u64, T), mpsc::RecvError> {
    self.recv_impl(self.rx.recv())
  }

  pub fn recv_mut(&mut self) -> Result<(u64, u64, T), mpsc::RecvError> {
    self.recv_mut_impl(self.rx.recv())
  }

  pub fn try_recv(&self) -> Result<(u64, u64, T), mpsc::TryRecvError> {
    self.recv_impl(self.rx.try_recv())
  }

  pub fn try_recv_mut(&mut self) -> Result<(u64, u64, T), mpsc::TryRecvError> {
    self.recv_mut_impl(self.rx.try_recv())
  }


  fn recv_impl<E>(&self, res: Result<(u64, T), E>) -> Result<(u64, u64, T), E> {
    match res {
      Ok((id, x)) => {
        let res = (self.callback_id, id, x);
        for (_, p) in self.proxy.iter() {
          p.send(res).unwrap_or(());
        }
        Ok(res)
      },
      Err(err) => Err(err),
    }
  }

  fn recv_mut_impl<E>(&mut self, res: Result<(u64, T), E>) -> Result<(u64, u64, T), E> {
    match res {
      Ok((id, x)) => {
        let res = (self.callback_id, id, x);
        let mut i = 0;
        while let Some((_, p)) = self.proxy.get(i) {
          match p.send(res) {
            Ok(()) => i+=1,
            Err(_) => { self.proxy.swap_remove(i); },
          }
        }
        Ok(res)
      },
      Err(err) => Err(err),
    }
  }

  pub fn iter(&self) -> Iter<T> {
    Iter { callback: self }
  }

  pub fn try_iter(&self) -> TryIter<T> {
    TryIter { callback: self }
  }

  pub fn iter_mut(&mut self) -> IterMut<T> {
    IterMut { callback: self }
  }

  pub fn try_iter_mut(&mut self) -> TryIterMut<T> {
    TryIterMut { callback: self }
  }
}


pub struct Iter<'a, T> {
  callback: &'a ScheduleCallback<T>,
}

pub struct IterMut<'a, T> {
  callback: &'a mut ScheduleCallback<T>,
}

pub struct TryIter<'a, T> {
  callback: &'a ScheduleCallback<T>,
}

pub struct TryIterMut<'a, T> {
  callback: &'a mut ScheduleCallback<T>,
}


impl<T: Copy> Iterator for Iter<'_, T> {
  type Item = (u64, u64, T);

  fn next(&mut self) -> Option<Self::Item> {
    self.callback.recv().ok()
  }
}

impl<T: Copy> Iterator for TryIter<'_, T> {
  type Item = (u64, u64, T);

  fn next(&mut self) -> Option<Self::Item> {
    self.callback.try_recv().ok()
  }
}

impl<T: Copy> Iterator for IterMut<'_, T> {
  type Item = (u64, u64, T);

  fn next(&mut self) -> Option<Self::Item> {
    self.callback.recv_mut().ok()
  }
}

impl<T: Copy> Iterator for TryIterMut<'_, T> {
  type Item = (u64, u64, T);

  fn next(&mut self) -> Option<Self::Item> {
    self.callback.try_recv_mut().ok()
  }
}


