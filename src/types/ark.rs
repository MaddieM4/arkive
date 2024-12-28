use crate::types::ipr::IPR;
use std::rc::Rc;

struct Ark<C, M=()> {
  paths: Rc<Vec<IPR>>,
  metas: Rc<Vec<M>>,
  files: Rc<Vec<C>>,
  links: Rc<Vec<String>>,
}

impl<C,M> Ark<C, M> {
  fn from_entries() -> Self {
    todo!()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_from_entries() {
    /*
    let ark = Ark.from_entries([
      ("", 
    ]);
    */
  }
}
