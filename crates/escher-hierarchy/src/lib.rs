pub mod x;
use std::slice;

pub use x::*;

/// Should be a sum type
pub trait Entity<Id, Input, Res, ResErr, H>: Sized where
  H: Hierarchy<Id, Self, Input, Res, ResErr>
{
  fn get_id(&self) -> Id;

  fn run(&self, hierarchy: &H, input: &Input) -> Res;
}



pub trait Hierarchy<Id, E, Input, Res, ResErr>: Sized where 
  E: Entity<Id, Input, Res, ResErr, Self>,
{
  fn iter_entities(&self) -> slice::Iter<E>;

  fn iter_mut_entities(&mut self) -> slice::IterMut<E>;

  fn access_entity(&mut self, id: &Id) -> Option<&mut E>;

  fn accumulate_results(&mut self, results: Vec<Res>) -> Result<Option<Input>, ResErr>;

  fn run(&mut self, input: Input) -> Result<(), ResErr> {
    let mut new_input = input;
    loop {
      let results = self.iter_entities()
        .map(|e| e.run(self, &new_input))
        .collect::<Vec<_>>();
      match self.accumulate_results(results) {
        Ok(None) => return Ok(()),
        Ok(Some(next_input)) => new_input = next_input,
        Err(e) => return Err(e),
      };
    }
  }

}


