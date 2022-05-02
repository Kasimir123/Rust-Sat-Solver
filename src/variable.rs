#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub value: Option<bool>,
    pub pos: usize,
    pub been_set: bool,
    pub last_set: bool,
}

impl Variable {
    pub fn new(name: String) -> Self {
        Variable {
            name,
            value: None,
            pos: !0,
            been_set: false,
            last_set: false,
        }
    }
}
