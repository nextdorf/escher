use std::{fmt::Debug, collections::{HashSet, HashMap}, hash::Hash};

/// Should be a sum type
pub trait Entity<Id, Input, State, Res, ResErr, H>: Sized where
  H: Hierarchy<Id, Self, Input, State, Res, ResErr>,
  Id: Sized + Hash + Eq + Debug,
{
  fn get_id(&self) -> Id;

  fn run(&mut self, state: &State, input: &Input) -> Option<Res>;
}



pub trait Hierarchy<Id, E, Input, State, Res, ResErr>: Sized where 
  E: Entity<Id, Input, State, Res, ResErr, Self>,
  Id: Sized + Hash + Eq + Debug,
{
  fn get_state(&self) -> &State;

  /// This function is potentially unsafe. It breaks rust's mutability model unless the data behind
  /// `get_state` and `update_entities` are independent. 
  fn map_entity_set<F>(&mut self, ids: &Option<HashSet<Id>>, input: &Input, f: F) -> Vec<Res> where F: Fn(&mut E, &State, &Input) -> Option<Res> {
    let state;
    let self_mut_res;
    unsafe {
      let self_ptr = self as *mut Self;
      state = self_ptr.as_ref().unwrap().get_state();
      self_mut_res = self_ptr.as_mut().unwrap();
    }
    match ids {
      // Some(ids) => self_mut_res.update_entities(|mut es| {
      //   let mut results = Vec::with_capacity(ids.len());
      //   for (id, e) in es.iter_mut() {
      //     if !ids.contains(&id) {
      //       continue;
      //     } else if let Some(next_res) = f(e, state, input) {
      //       // results.push(e.run(state, &input))
      //       results.push(next_res);
      //     }
      //   }
      //   (results, es)
      // }),
      Some(ids) => self_mut_res.update_entities(|mut es| {
        let mut results = Vec::with_capacity(ids.len());
        for id in ids.iter() {
          if let Some(e) = es.get_mut(id) {
            if let Some(next_res) = f(e, state, input) {
              // results.push(e.run(state, &input))
              results.push(next_res);
            }
          }
        }
        (results, es)
      }),
      None => self_mut_res.update_entities(|mut es| {
        let mut results = Vec::with_capacity(es.len());
        for e in es.values_mut() {
          if let Some(next_res) = f(e, state, input) {
            // results.push(e.run(state, &input))
            results.push(next_res);
          }
        }
        (results, es)
      }),
    }
  }

  // fn iter_mut_entities(&mut self) -> collections::hash_map::IterMut<Id, E>;
  /// Updates the `entities` attribute with `f: F`. `f` takes ownership of the entities, updates
  /// the map and returns it together with a result. If the map is just represented by a
  /// hashmap member, the function can be written like that:
  /// ```
  /// fn update_entities<F, G>(&mut self, f: F) -> G where F: Fn(HashMap<Id, E>) -> (G, HashMap<Id, E>) {
  ///     let res;
  ///     let entities = std::mem::take(&mut self.entities);
  ///     (res, self.entities) = f(entities);
  ///     res
  /// }
  /// ```
  fn update_entities<F, G>(&mut self, f: F) -> G where F: Fn(HashMap<Id, E>) -> (G, HashMap<Id, E>);

  fn access_entity(&mut self, id: &Id) -> Option<&mut E>;

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
}




