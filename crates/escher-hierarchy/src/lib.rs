use std::{fmt::Debug, collections::{HashSet, HashMap}, hash::Hash};

/// Should be a sum type
pub trait Entity<Id, Input, State, Res>: Sized where
  Id: Sized + Hash + Eq + Debug,
{
  fn get_id(&self) -> Id;

  fn run(&mut self, state: &State, input: &Input) -> Option<Res>;
}


pub enum InteriorKind {
  None,
  AsRef,
  AsMut,
  Owning,
}

pub enum InteriorRef<'a, T> {
  None,
  AsRef(&'a T),
  AsMut(&'a mut T),
  Owning(T),
}

impl<'a, T> InteriorRef<'a, T> {
  pub fn as_ref(&'a self) -> Option<&'a T> {
    match self {
      InteriorRef::AsRef(r) => Some(r),
      InteriorRef::AsMut(r) => Some(r),
      InteriorRef::Owning(x) => Some(x),
      InteriorRef::None => None,
    }
  }
  pub fn as_mut<'b: 'a>(&'b mut self) -> Option<&'a mut T> {
    match self {
      InteriorRef::AsMut(r) => Some(r),
      InteriorRef::Owning(x) => Some(x),
      _ => None
    }
  }
  pub fn to_mut(self) -> Option<&'a mut T> {
    match self {
      InteriorRef::AsMut(r) => Some(r),
      _ => None
    }
  }
  pub fn own(self) -> Option<T> {
    match self {
      InteriorRef::Owning(x) => Some(x),
      _ => None
    }
  }
}


pub trait Hierarchy<Id, E, Input, State, Res, ResErr>: Sized where 
  E: Entity<Id, Input, State, Res>,
  Id: Sized + Hash + Eq + Debug,
{
  fn represent(&self, state_kind: InteriorKind, entities_kind: InteriorKind) -> (InteriorRef<State>, InteriorRef<HashMap<Id, E>>);
  fn represent_mut<'a, 'b, 'c: 'a + 'b>(&'c mut self, state_kind: InteriorKind, entities_kind: InteriorKind) -> (InteriorRef<'a, State>, InteriorRef<'b, HashMap<Id, E>>);

  fn map_entity_set<F>(&mut self, ids: &Option<HashSet<Id>>, input: &Input, f: F) -> Vec<Res> where F: Fn(&mut E, &State, &Input) -> Option<Res> {
    let (interior_state, interior_entities) = self.represent_mut(InteriorKind::AsRef, InteriorKind::AsMut);
    let (state, entities) = (interior_state.as_ref().unwrap(), interior_entities.to_mut().unwrap());
    
    let mut es = std::mem::take(entities);
    let mut results;
    match ids {
      Some(ids) => {
        results = Vec::with_capacity(ids.len());
        for id in ids.iter() {
          if let Some(e) = es.get_mut(id) {
            if let Some(next_res) = f(e, state, input) {
              results.push(next_res);
      }}}},
      None => {
        results = Vec::with_capacity(es.len());
        for e in es.values_mut() {
          if let Some(next_res) = f(e, state, input) {
            results.push(next_res);
      }}},
    }
    *entities = es;
    results
  }


  fn access_entity<'a, 'b: 'a, 'c: 'a>(&'c mut self, id: &'b Id) -> Option<&'a mut E> {
    let entity_repr = self.represent_mut(InteriorKind::AsRef, InteriorKind::AsMut).1;
    match entity_repr.to_mut() {
      Some(entity_mut) => entity_mut.get_mut(id),
      None => None
    }
  }

  fn accumulate_results(&mut self, results: Vec<Res>) -> Result<Option<(Option<HashSet<Id>>, Input)>, ResErr>;

  fn run(&mut self, ids: Option<HashSet<Id>>, input: Input) -> Result<(), ResErr> {
    let mut ids = ids;
    let mut input = input;
    loop {
      let update_entities_res = self.map_entity_set(&ids, &input, |e, state, input| e.run(state, input));
      match self.accumulate_results(update_entities_res) {
        Ok(Some((new_ids, new_input))) => {
          ids = new_ids;
          input = new_input;
        },
        Ok(None) => return Ok(()),
        Err(err) => return Err(err),
      }
    }
  }
} //TODO: Add FullInput and FullOutput and move run out of the function for reference


