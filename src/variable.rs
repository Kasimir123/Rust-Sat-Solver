#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub maybe_value: Option<bool>,
}

impl Variable {
    pub fn new(name: String) -> Self {
        Variable {
            name,
            maybe_value: None,
        }
    }
}
