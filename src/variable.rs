use std::cell::Cell;

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub maybe_value: Cell<Option<bool>>,
}

impl Variable {
    pub fn new(name: String) -> Self {
        Variable {
            name,
            maybe_value: Cell::new(None),
        }
    }
}
