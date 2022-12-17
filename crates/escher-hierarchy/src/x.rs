use std::collections::HashMap;

use super::*;

#[derive(Debug)]
pub enum XBase {
  Add(usize),
  Mult(usize),
  C(usize, usize),
  Noop,
}
#[derive(Debug)]
pub struct X { id: usize, pub val: XBase }

#[derive(Debug, Default)]
pub struct HX { next_id: usize, id_to_idx: HashMap<usize, usize>, xs: Vec<X> }


impl Entity<usize, usize, (), (), HX> for X {
  fn get_id(&self) -> usize {
    self.id
  }

  fn run(&self, _hx: &HX, _input: &usize) {
    // self.1 = match self.1 {
    //   XBase::Add(x) => XBase::Add(x+input),
    //   XBase::Mult(x) => XBase::Mult(x*input),
    //   XBase::C(x, n) => XBase::C((x+1) % n, n),
    //   XBase::Noop => XBase::Noop,
    // }
  }
}


impl Hierarchy<usize, X, usize, (), ()> for HX {
  fn iter_entities(&self) -> slice::Iter<X> {
    self.xs.iter()
  }

  fn iter_mut_entities(&mut self) -> slice::IterMut<X> {
    self.xs.iter_mut()
  }

  fn access_entity(&mut self, id: &usize) -> Option<&mut X> {
    self.xs.get_mut(*(self.id_to_idx.get(id)?))
  }

  fn accumulate_results(&mut self, _results: Vec<()>) -> Result<Option<usize>, ()> {
    Ok(None)
  }
}


impl HX {
  pub fn new(&mut self, val: XBase) -> X {
    self.next_id += 1;
    X { id: self.next_id, val}
  }
  
  pub fn add_new(&mut self, val: XBase) -> &X {
    let new_x = self.new(val);
    let id = new_x.id;
    let idx = self.xs.len();
    self.xs.push(new_x);
    if let Some(_colliding_idx) = self.id_to_idx.insert(id, idx) {
      panic!("Id collision for {id}");
    }
    self.xs.last().unwrap()
  }
  
}

