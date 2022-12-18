use std::{collections::{HashMap, HashSet}, fmt};

use super::*;

pub enum XType {
  Add {res: f64, val: f64},
  Mult {res: f64, val: f64},
  Count(usize),
  Noop,
}

impl fmt::Debug for XType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Add { res, val } => f.write_fmt(format_args!("[{} <- x + {}]", res, val)),
      Self::Mult { res, val } => f.write_fmt(format_args!("[{} <- x * {}]", res, val)),
      Self::Count(arg0) => f.write_fmt(format_args!("[Count = {}]", arg0)),
      Self::Noop => write!(f, "Noop"),
    }
  }
}

#[derive(Debug)]
pub struct X { id: usize, pub val: XType }

#[derive(Default)]
pub struct HX { next_id: usize, entities: HashMap<usize, X> }

impl fmt::Debug for HX {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("HX")
      .field("next_id", &self.next_id)
      .field("entities", &self.entities
        .iter()
        .map(|(id, X {id: _, val})| (id, val))
        .collect::<HashMap<_, _>>()
      ).finish()
  }
}

impl Entity<usize, f64, (), f64, (), HX> for X {
  fn get_id(&self) -> usize {
    self.id
  }

  fn run(&mut self, _state: &(), input: &f64) -> Option<f64> {
    match self.val {
      XType::Add { res: _, val } => {
        let res = input+val;
        self.val = XType::Add { res, val };
        Some(res)
      },
      XType::Mult { res: _, val } => {
        let res = input*val;
        self.val = XType::Mult { res, val };
        Some(res)
      },
      XType::Count(n) => {
        self.val = XType::Count(n+1);
        None
      },
      XType::Noop => None,
    }
  }
}


impl Hierarchy<usize, X, f64, (), f64, ()> for HX {
  fn get_state(&self) -> &() {
    &()
  }

  fn update_entities<F, G>(&mut self, f: F) -> G where F: Fn(HashMap<usize, X>) -> (G, HashMap<usize, X>) {
    let res;
    let entities = std::mem::take(&mut self.entities);
    (res, self.entities) = f(entities);
    res
  }

  fn access_entity(&mut self, id: &usize) -> Option<&mut X> {
    self.entities.get_mut(id)
  }

  fn accumulate_results(&mut self, _results: Vec<f64>) -> Result<Option<(Option<HashSet<usize>>, f64)>, ()> {
    Ok(None)
  }
}


impl HX {
  fn inc_id(&mut self) -> usize {
    let ret = self.next_id;
    self.next_id += 1;
    ret
  }
  pub fn add_new(&mut self, x: XType) {
    let id = self.inc_id();
    if self.entities.insert(id, X { id, val: x }).is_some() {
      panic!("Id collision for {id}");
    }
  }
}

