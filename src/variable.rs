#[derive(Eq)]
#[derive(Hash)]
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
impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
