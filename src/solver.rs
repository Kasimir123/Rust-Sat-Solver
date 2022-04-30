// import out structs
use crate::connections::{Connection, ConnectionGroup};
use crate::variable::Variable;

// import required imports
use std::error::Error;

use std::io::{BufRead, BufReader, Read};

pub struct SolveResult {
    pub sat: bool,
    pub connections_checked: u64,
    pub num_backtracks: u64,
}

#[derive(Default)]
pub struct Solver {
    // variables in the solver
    variables: Vec<Variable>,

    // connections in the solver
    connection_groups: Vec<ConnectionGroup>,

    connections: Vec<Connection>,

    // number of backtracks
    backtracks: usize,

    // number of connections checked
    connections_checked: usize,
}

impl Solver {
    // create a new solver instance
    pub fn new() -> Self {
        Solver {
            variables: Vec::new(),
            connection_groups: Vec::new(),
            connections: Vec::new(),
            backtracks: 0,
            connections_checked: 0,
        }
    }

    // adds a variable to the solver, if the variable exists then return its position,
    // otherwise add it and return the position
    pub fn add_variable(&mut self, name: String) -> Option<usize> {
        for i in 0..self.variables.len() {
            if self.variables[i].name.eq(&name) {
                return Some(i);
            }
        }

        let mut new_var = Variable::new(name);
        new_var.pos = self.variables.len();

        self.variables.push(new_var);

        Some(self.variables.len() - 1)
    }

    // loads the standard cnf benchmark file into the solver
    pub fn load_cnf(&mut self, source: impl Read) -> Result<(), Box<dyn Error>> {
        let buf_reader = BufReader::new(source);
        let mut check = false;
        for maybe_line in buf_reader.lines() {
            let line = maybe_line?;
            if line.contains("p cnf") {
                check = true;
            } else if line.contains('%') {
                return Ok(());
            } else if check {
                let st = line.trim();
                let st = st.split(' ');
                let st: Vec<&str> = st.collect();

                let mut con_group = ConnectionGroup::default();

                for i in 0..3 {
                    let mut var_name = st[i];
                    let neg = var_name.contains('-');
                    if neg {
                        var_name = &var_name[1..];
                    }

                    let var_pos = self.add_variable(var_name.to_owned()).unwrap();
                    let connection = Connection::new(var_pos, !neg);
                    self.connections.push(connection);

                    con_group.connections.push(self.connections.len() - 1);
                }

                self.connection_groups.push(con_group);
            }
        }
        Ok(())
    }

    // checks an individual connection
    pub fn check_connection(&self, connection: usize) -> Option<bool> {
        let connection = self.connections.get(connection)?;

        let var = self.variables.get(connection.var_pos)?;

        let ret = match var.value {
            None => Some(true),
            _ => Some(false),
        };

        if ret? {
            return Some(true);
        }

        Some(var.value? == connection.val)
    }

    // solves the sat problem
    pub fn solve(&mut self) -> SolveResult {
        // initialize a vector to hold assigned values
        let mut assigned = Vec::new();

        // push the first value into the assigned values
        assigned.push(self.variables[assigned.len()].pos);

        // while we have at least one value to be assigned
        while !assigned.is_empty() {
            // if everything is assigned then return true
            if assigned.len() >= self.variables.len() {
                return SolveResult {
                    sat: true,
                    connections_checked: self.connections_checked as u64,
                    num_backtracks: self.backtracks as u64,
                };
            }

            // gets the variable to assigned
            let cur = self.variables.get(*assigned.last().unwrap()).unwrap();

            // gets the variables position
            let pos = cur.pos;

            let new_val = match cur.value {
                None => Some(true),
                Some(true) => Some(false),
                Some(false) => unreachable!(),
            };

            self.variables[pos].value = new_val;

            // loop through connections and perform out checks

            let mut connections_checked = 0;
            let check = self.connection_groups.iter().all(|group| {
                let or_check = group.connections.iter().any(|con| {
                    connections_checked += 1;
                    self.check_connection(*con as usize).unwrap()
                });

                or_check
            });

            self.connections_checked += connections_checked;

            // if check is true, push the next variable to be assigned
            if check {
                assigned.push(self.variables[assigned.len()].pos);
            }
            // else, if the value was false, go through and backtrack
            else {
                while matches!(
                    self.variables.get(*assigned.last().unwrap()).unwrap().value,
                    Some(false)
                ) {
                    let assigned_last = assigned.pop().unwrap();
                    self.variables[assigned_last].value = None;
                    self.backtracks += 1;
                }
            }
        }

        // return false if unsat
        SolveResult {
            sat: false,
            connections_checked: self.connections_checked as u64,
            num_backtracks: self.backtracks as u64,
        }
    }

    // prints out the variables
    pub fn print_variables(&self) {
        for var in &self.variables {
            println!("{} {}", var.name, var.value.unwrap());
        }
    }
}
