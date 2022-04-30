pub struct Connection {
    pub val: bool,
    pub var_pos: usize,
}

impl Connection {
    pub fn new(pos: usize, val: bool) -> Self {
        Connection { val, var_pos: pos }
    }
}

#[derive(Default)]
pub struct ConnectionGroup {
    pub connections: Vec<usize>,
}
