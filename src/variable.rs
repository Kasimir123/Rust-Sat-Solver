pub struct Variable {
    pub name: String,
    pub value: Option<bool>,
    pub has_been_set_before: Option<bool>,
    pub pos: usize,
}

impl Variable {
    pub fn new(name: String) -> Self {
        Variable {
            name,
            value: None,
            has_been_set_before: None,
            pos: !0,
        }
    }
}
