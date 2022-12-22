use std::{fmt::Debug, collections::{HashSet, HashMap}, hash::Hash};

/// Should be a sum type
pub trait Entity<Id, Input, State, Res>: Sized where
  Id: Sized + Hash + Eq + Debug,
{
  fn get_id(&self) -> Id;

  fn run(&mut self, state: &State, input: &Input) -> Option<Res>;
} // TODO: I had an idea about allowing several Entity traits for the same hierarchy, but I forgot it.


/// Describes how caller wants to access the interior data
pub enum InteriorKind {
  /// Caller doesn't care about the data
  None,
  /// Caller wishes to access the data with an immutable ref
  AsRef,
  /// Caller wishes to access the data with a mutable ref
  AsMut,
  /// Caller wishes to own the data
  Owning,
}

/// Describes how callee can represent interior data. In terms of accessibility `AsMut` > `Owning` > `AsRef` > `None`.
pub enum InteriorRef<'a, T> {
  /// Callee forbids access to the data.
  None,
  /// Callee allows for immutable acces to the data.
  AsRef(&'a T),
  /// Callee allows for mutable acces to the data.
  AsMut(&'a mut T),
  /// Callee gave up ownership over the data. `InteriorRef` object is now the owner.
  Owning(T),
}

impl<'a, T> InteriorRef<'a, T> {
  /// Tries to access data as an immutable reference. Possible for `AsRef`, `AsMut`, `Owning`. Otherwise `None`
  pub fn as_ref(&'a self) -> Option<&'a T> {
    match self {
      InteriorRef::AsRef(r) => Some(r),
      InteriorRef::AsMut(r) => Some(r),
      InteriorRef::Owning(x) => Some(x),
      InteriorRef::None => None,
    }
  }
  /// Tries to access data as a mutable reference. Possible for `AsMut`, `Owning`. Otherwise `None`
  pub fn as_mut<'b: 'a>(&'b mut self) -> Option<&'a mut T> {
    match self {
      InteriorRef::AsMut(r) => Some(r),
      InteriorRef::Owning(x) => Some(x),
      _ => None
    }
  }
  /// Tries to cast the InteriorRef to a mutable reference. Possible for `AsMut`. Otherwise `None`
  pub fn to_mut(self) -> Option<&'a mut T> {
    match self {
      InteriorRef::AsMut(r) => Some(r),
      _ => None
    }
  }
  /// Tries to cast the InteriorRef to the underlying data. Possible for `Owning`. Otherwise `None`
  pub fn own(self) -> Option<T> {
    match self {
      InteriorRef::Owning(x) => Some(x),
      _ => None
    }
  }
}


pub trait Hierarchy<Id, E, Input, FullInput, State, Res, FullRes, ResErr>: Sized where 
  E: Entity<Id, Input, State, Res>,
  Id: Sized + Hash + Eq + Debug,
{
  /// Attempts to represent access to the interior data which is at least as accessable as what the
  /// caller is asking for. If a more accessable InteriorRef can be returned without overhead, it
  /// should be done.
  fn represent(&self, state_kind: InteriorKind, entities_kind: InteriorKind) -> (InteriorRef<State>, InteriorRef<HashMap<Id, E>>);

  /// Attempts to represent access to the interior data which is at least as accessable as what the
  /// caller is asking for. If a more accessable InteriorRef can be returned without overhead, it
  /// should be done.
  fn represent_mut<'a, 'b, 'c: 'a + 'b>(&'c mut self, state_kind: InteriorKind, entities_kind: InteriorKind) -> (InteriorRef<'a, State>, InteriorRef<'b, HashMap<Id, E>>);


  /// Applies `f` to all entities in the hierarchy system if `ids` is `None`. If `ids` is `Some`
  /// only the entities with an id in `ids` are mapped. Either way all successful results are
  /// pushed to a vector which will be returened. The order of the results is an implemntation
  /// detail of `HashMap` and `HashSet`.
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


  /// Tries to access entity with given `id`. Fails if entity can't be represented as mutable or
  /// if `id` wasn't found.
  fn access_entity<'a, 'b: 'a, 'c: 'a>(&'c mut self, id: &'b Id) -> Option<&'a mut E> {
    let entity_repr = self.represent_mut(InteriorKind::AsRef, InteriorKind::AsMut).1;
    match entity_repr.to_mut() {
      Some(entity_mut) => entity_mut.get_mut(id),
      None => None
    }
  }

  /// The function which takes the results of a run and prepares the next run. If `Ok(None)` is
  /// returned the entire run is completed. If `Ok(Some(..))` is returned then there should be another
  /// run with `(..)` being the arguments for the next run. If `Err(ResErr)` is returned an error
  /// occurred
  fn accumulate_results(&mut self, results: Vec<Res>) -> Result<Option<(Option<HashSet<Id>>, FullInput)>, ResErr>;

  /// The running function of the entire hierarchy. Should do something like `prepare` - 
  /// `map FullInput to Input` - `call map_entity_set` - `call accumulate_results` - either
  /// `clean up and done` or `next run`. See `run_hierarchy_default` for a simplified default
  /// implementation.
  fn run(&mut self, ids: Option<HashSet<Id>>, input: FullInput) -> Result<FullRes, ResErr>;

}

/// Default implementation for Hierarchy<Id, E, Input, Input, State, Res, (), ResErr>::run
pub fn run_hierarchy_default<H, Id, E, Input, State, Res, ResErr>(this: &mut H, ids: Option<HashSet<Id>>, input: Input) -> Result<(), ResErr>
  where H : Hierarchy<Id, E, Input, Input, State, Res, (), ResErr>, E: Entity<Id, Input, State, Res>, Id: Sized + Hash + Eq + Debug
{
  let mut ids = ids;
  let mut input = input;
  loop {
    let update_entities_res = this.map_entity_set(&ids, &input, |e, state, input| e.run(state, input));
    match this.accumulate_results(update_entities_res) {
      Ok(Some((new_ids, new_input))) => {
        ids = new_ids;
        input = new_input;
      },
      Ok(None) => return Ok(()),
      Err(err) => return Err(err),
    }
  }
}

