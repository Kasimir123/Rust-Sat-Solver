pub struct Variable {
    pub name: String,
    pub value: bool,
    pub is_set: bool,
    pub pos: usize,
}

impl Variable {
    pub fn new(name: String) -> Self {
        Variable {
            name: name,
            value: true,
            is_set: false,
            pos: !0,
        }
    }
}
