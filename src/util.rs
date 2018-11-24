pub trait FilterableResult<T, E> {
  fn filter<P: FnOnce(&T) -> bool>(self, predicate: P, error: E) -> Result<T, E>;
}

impl<T, E> FilterableResult<T, E> for Result<T, E> {
  fn filter<P: FnOnce(&T) -> bool>(self, predicate: P, error: E) -> Result<T, E> {
    match self {
      Ok(x) => if predicate(&x) {
        return Ok(x);
      } else {
        return Err(error);
      },
      Err(x) => Err(x),
    }
  }
}
