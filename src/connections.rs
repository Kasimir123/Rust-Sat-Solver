

#[derive(Debug)]
pub struct Connection {
    pub val: bool,
    pub var_pos: usize,
}

impl Connection {
    pub fn new(pos: usize, val: bool) -> Self {
        Connection { val, var_pos: pos }
    }
}

#[derive(Debug)]
pub struct ConnectionGroup {
    pub connections: Vec<usize>,
    pub sat: bool
}

impl Default for ConnectionGroup {
    fn default() -> Self {
        ConnectionGroup {
            connections: Vec::new(),
            sat: false
        }
    }
}
