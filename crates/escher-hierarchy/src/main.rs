mod x;
use x::*;

use escher_hierarchy::*;


fn main() {
  let mut hierarchy = HX::default();
  println!("{hierarchy:?}");
  hierarchy.add_new(XType::Add { res: 0., val: 5. });
  println!("{hierarchy:?}");
  hierarchy.add_new(XType::Noop);
  println!("{hierarchy:?}");
  hierarchy.run(None, 42.).unwrap();
  println!("{hierarchy:?}");

  hierarchy.add_new(XType::Mult { res: 1., val: 2. });
  println!("{hierarchy:?}");
  hierarchy.add_new(XType::Count(10));
  println!("{hierarchy:?}");

  hierarchy.run(None, 42.).unwrap();
  println!("{hierarchy:?}");
  hierarchy.run(None, 42.).unwrap();
  println!("{hierarchy:?}");
  hierarchy.run(None, 42.).unwrap();
  println!("{hierarchy:?}");
}

