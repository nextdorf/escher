use std::{sync::{Arc, Mutex, mpsc, Weak}, thread::JoinHandle};


pub enum RequestKind {
  Plain,
  Once(Arc<Mutex<bool>>),
}
pub enum Response<T> {
  Init,
  Ok(T),
  // Pong(usize),
  Ready(bool),
}

pub enum BroadcastKind {
  Specific(usize),
  All,
  DoNow,
  TryNow,
  MulipleTimes(usize),
}
pub enum RequestError<T> {
  IndexInvalid(T),
  WorkerDied(T), 
  SendError(T),
  NoReadyWorkers(T)
}

pub struct Scheduler<Req, T> {
  workers: Vec<Option<WorkerHandle<Req, T>>>,
  weak_ref: Weak<Self>,
  // next_ping_id: usize, //ping pong was a nonsene idea
}

struct WorkerHandle<Request, T> {
  tx: mpsc::Sender<(RequestKind, Request)>,
  rx: mpsc::Receiver<Response<T>>,
  is_ready: bool,
  thread: JoinHandle<()>
}

// struct Worker<Request, T> {
//   thread_idx: usize,
//   rx: mpsc::Receiver<(BroadcastKind, Request)>,
//   tx: mpsc::Sender<Response<T>>,
// }


impl<Req, T> Scheduler<Req, T> where Req: Send + Clone, T: Send {
  pub fn new<W, F>(num_workers: usize, worker_init: F) -> Arc<Self>
    where W: Worker<Req, T>, F:Fn(usize) -> W + Send + Sync + 'static, Req: 'static, T: 'static
  {
    let mut workers = Vec::with_capacity(num_workers);
    let worker_init = Arc::new(worker_init);
    for i in 0..num_workers {
      let (t_resp, r_resp) = mpsc::channel();
      let (t_req, thread) = new_worker(i, t_resp, worker_init.clone());
      workers.push(Some(WorkerHandle {tx: t_req, rx: r_resp, is_ready: false, thread}));
    }
    
    Arc::new_cyclic(|weak_ref| {
      Self { workers, weak_ref: weak_ref.clone() }
    })
  }


  pub fn request(&self, request: Req, kind: BroadcastKind) -> Result<(), RequestError<Req>> {
    match kind {
      BroadcastKind::Specific(i) => match self.workers.get(i) {
        Some(Some(w)) => w.just_send(request),
        Some(None) => Err(RequestError::WorkerDied(request)),
        None => Err(RequestError::IndexInvalid(request)),
      },
      BroadcastKind::DoNow => {
        for w in self.workers.iter() {
          if let Some(w) = w {
            if w.is_ready {
              return w.just_send(request);
            }
          }
        }
        Err(RequestError::NoReadyWorkers(request))
      },
      BroadcastKind::TryNow => {
        for w in self.workers.iter() {
          if let Some(w) = w {
            if w.is_ready {
              return w.just_send(request);
            }
          }
        }
        Ok(())
      },
      BroadcastKind::All => {
        for w in self.workers.iter() {
          if let Some(w) = w {
            if let Err(err) = w.just_send(request.clone()) {
              return Err(err);
            }
          }
        }
        Ok(())
      },
      BroadcastKind::MulipleTimes(n) => {
        let mut mutexes = Vec::with_capacity(n);
        for _ in 0..n {
          mutexes.push(Arc::new(Mutex::new(true)))
        }
        for w in self.workers.iter() {
          if let Some(w) = w {
            for m in mutexes.iter() {
              if let Err(err) = w.send(RequestKind::Once(m.clone()), request.clone()) {
                return Err(err);
              }
            }
          }
        }
        Ok(())
      },
    }
  }

  pub fn handle_respones<E>(&mut self, f: fn(Response<T>, &mpsc::Sender<(RequestKind, Req)>) -> Result<(), E>) -> Result<(), E> {
    for w in self.workers.iter_mut() {
      if let Some(WorkerHandle { tx, rx, is_ready, thread }) = w {
        if thread.is_finished() {
          *w = None;
          continue;
        } else if *is_ready {
          continue;
        } else {
          match rx.try_recv() {
            Ok(first_val) => {
              let iter = std::iter::once(first_val).chain(rx.try_iter());
              for w in iter {
                f(w, tx)?;
              }
            },
            Err(mpsc::TryRecvError::Disconnected) => {*w = None},
            Err(mpsc::TryRecvError::Empty) => {},
          }
        }
      }
    }
    Ok(())
  }

}

pub trait Worker<Req, T> where Req: Send, T: Send {
  fn handle(&mut self, request: Req, kind: RequestKind) -> Response<T>;
}

impl<Req, T, F> Worker<Req, T> for F where F: FnMut(Req, RequestKind) -> Response<T>, Req: Send, T: Send {
  fn handle(&mut self, request: Req, kind: RequestKind) -> Response<T> {
    self(request, kind)
  }
}

pub fn new_worker<Req, T, W, F>(thread_idx: usize, t_resp: mpsc::Sender<Response<T>>, worker_init: Arc<F>) -> (mpsc::Sender<(RequestKind, Req)>, JoinHandle<()>)
  where F: Fn(usize) -> W + Send + Sync + 'static, W: Worker<Req, T>, Req: Send + 'static, T: Send + 'static
{
  let (tx, rx) = mpsc::channel();
  let t = std::thread::spawn(move || {
    let (t_req, r_req) = mpsc::channel::<(RequestKind, Req)>();
    tx.send(t_req).unwrap();
    let mut worker = worker_init(thread_idx);
    drop(worker_init);
    t_resp.send(Response::Init).unwrap();
    // let mut worker = Self { thread_idx, rx: r_req, tx: t_resp, state: worker_state};

    loop {
      match r_req.recv() {
        Ok((kind, req)) => match t_resp.send(worker.handle(req, kind)) {
          Ok(()) => {},
          Err(e) => eprintln!("Response Error: {:?}", e)
        },
        Err(e) => eprintln!("Request Error: {:?}", e)
      }
    }
  });
  (rx.recv().unwrap(), t)
}

impl<Req, T> WorkerHandle<Req, T> {
  pub fn send(&self, kind: RequestKind, request: Req) -> Result<(), RequestError<Req>> {
    match self.tx.send((kind, request)) {
      Ok(()) => Ok(()),
      Err(e) => Err(e.into())
    }
  }

  pub fn just_send(&self, request: Req) -> Result<(), RequestError<Req>> {
    self.send(RequestKind::Plain, request)
  }
}



impl<Req, T> From<mpsc::SendError<(T, Req)>> for RequestError<Req> {
  fn from(value: mpsc::SendError<(T, Req)>) -> Self {
    RequestError::SendError(value.0.1)
  }
}



