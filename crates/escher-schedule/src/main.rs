use std::sync::mpsc;

use escher_schedule::*;

fn main() {
  match 1 {
    0 => test_split_last_iter(),
    1 => test_schedule(),
    _ => ()
  }
}

fn test_split_last_iter() {
  //let numbers = vec![1,2,3,4];
  let numbers: Vec<usize> = vec![1];
  println!("Numbers: {:?}", numbers);
  let mut split_iter = SplitLastIter::from_iter(numbers.iter());
  println!("Last One: {:?}", split_iter.get_last());
  loop {
    if let Some(i) = split_iter.next() {
      println!("i: {}", i);
    } else {
      break;
    }
  }
  println!("Last One: {:?}", split_iter.get_last());
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DummyReq {
  Text(&'static str),
  Idx,
  Kill,
}

#[derive(Debug, Clone, Copy)]
enum DummyMsg {
  Text(&'static str),
  Idx(usize),
}

fn test_schedule() {
  let mut scheduler = Scheduler::new(1, DummyReq::Kill, |worker_idx| {
    let worker = move |request: DummyReq, kind: RequestKind, pub_tx: &mut CallbackSender<DummyMsg>| {
      match kind {
        RequestKind::Plain => (),
        RequestKind::Once(m) => {
          let mut guard = m.lock().unwrap();
          if *guard {
            *guard = false;
          } else {
            return Response::Ok(());
          }
        },
      }
      match request {
        DummyReq::Text(s) => println!("{}", s),
        DummyReq::Idx => pub_tx.send(&DummyMsg::Idx(worker_idx)),
        DummyReq::Kill => println!("Killing worker-thread {} requested", worker_idx),
      }
      Response::Ok(())
    };
    worker
  });

  scheduler.request(DummyReq::Text("Moin"), BroadcastKind::All).unwrap();
  scheduler.request(DummyReq::Text("1"), BroadcastKind::MulipleTimes(1)).unwrap();
  scheduler.request(DummyReq::Text("2"), BroadcastKind::MulipleTimes(2)).unwrap();
  scheduler.request(DummyReq::Text("3"), BroadcastKind::MulipleTimes(3)).unwrap();

  fn handle_respones(_resp: Response<()>, _tx: &mpsc::Sender<(RequestKind, DummyReq)>) -> Result<(), ()> {
    Ok(())
  }
  scheduler.handle_respones(handle_respones).unwrap();
  // scheduler.kill_all_workers(false).unwrap();
  scheduler.kill_all_workers(true).unwrap();
}

