use escher_hierarchy::*;


fn main() {
  let mut hierarchy = HX::default();
  println!("{hierarchy:?}");
  hierarchy.add_new(XBase::Add(5));
  println!("{hierarchy:?}");
  hierarchy.add_new(XBase::Noop);
  println!("{hierarchy:?}");
  hierarchy.run(16).unwrap();
  println!("{hierarchy:?}");
}

