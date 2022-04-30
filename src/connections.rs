pub struct Connection {
    pub val: bool,
    pub var_pos: usize,
}

impl Connection {
    pub fn new(pos: usize, val: bool) -> Self {
        Connection {
            val: val,
            var_pos: pos,
        }
    }
}

pub struct ConnectionGroup {
    pub connections: Vec<Connection>,
}

impl ConnectionGroup {
    pub fn new() -> Self {
        ConnectionGroup {
            connections: Vec::new(),
        }
    }
}
