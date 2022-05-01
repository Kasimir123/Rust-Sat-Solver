#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub value: Option<bool>,
    pub pos: usize,
}

impl Variable {
    pub fn new(name: String) -> Self {
        Variable {
            name,
            value: None,
            pos: !0,
        }
    }
}
