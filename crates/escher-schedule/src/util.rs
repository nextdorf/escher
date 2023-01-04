
pub struct SplitLastIter<T, Iter> where Iter: Iterator<Item=T> {
  inner: Iter,
  next_one: Option<T>,
  last_one: Option<T>,
}


impl<T, Iter> SplitLastIter<T, Iter> where Iter: IntoIterator<Item=T> + Iterator<Item=T> {
  pub fn from_iter(mut iter: Iter) -> Self {
    match iter.next() {
      Some(first) => Self { inner: iter, next_one: Some(first), last_one: None },
      None => Self { inner: iter, next_one: None, last_one: None }
    }
  }

  pub fn get_last(&self) -> &Option<T> { &self.last_one }

  pub fn get_mut_last(&mut self) -> &mut Option<T> { &mut self.last_one }

  pub fn unwrap_last(self) -> Option<T> {
    match self.last_one {
      Some(last) => Some(last),
      None => self.inner.last(),
    }
  }

}

impl<T, Iter> Iterator for SplitLastIter<T, Iter> where Iter: Iterator<Item=T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    let ret;
    match self.inner.next() {
      Some(new_next_one) => ret = std::mem::replace(&mut self.next_one, Some(new_next_one)),
      None => {
        ret = None;
        if let Some(actual_last_one) = std::mem::take(&mut self.next_one) {
          self.last_one = Some(actual_last_one);
        }
      },
    }
    ret
  }

  

}

