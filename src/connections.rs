use std::{cell::RefCell, rc::Rc};

use crate::variable::Variable;

#[derive(Debug)]
pub struct Connection {
    val: bool,
    variable: Rc<RefCell<Variable>>,
}

impl Connection {
    pub fn new(variable: Rc<RefCell<Variable>>, val: bool) -> Self {
        Connection { val, variable }
    }
}

pub struct GroupCheckResult {
    pub success: bool,
    pub connections_checked: u64,
}

#[derive(Default)]
pub struct ConnectionGroup {
    connections: Vec<Connection>,
}

impl ConnectionGroup {
    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    pub fn num_connections(&self) -> usize {
        self.connections.len()
    }

    pub fn check_group(&self) -> GroupCheckResult {
        let mut connections_checked = 0;
        for connection in self.connections.iter() {
            connections_checked += 1;

            if connection
                .variable
                .borrow()
                .maybe_value
                .map_or(true, |variable_value| variable_value == connection.val)
            {
                return GroupCheckResult {
                    success: true,
                    connections_checked,
                };
            }
        }

        GroupCheckResult {
            success: false,
            connections_checked,
        }
    }
}
