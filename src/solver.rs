// import out structs
use crate::connections::{Connection, ConnectionGroup};
use crate::variable::Variable;

// import required imports
use std::fs::File;
use std::path::Path;
use std::error::Error;

use std::io::{BufRead, BufReader, Read};
use std::rc::Rc;

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
    connections: Vec<ConnectionGroup>,

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

    // gets a variable from the variables based on position
    pub fn get_variable(&self, pos: usize) -> Option<&Variable> {
        return Some(&self.variables[pos]);
    }

    // sets a variable from the variables to the passed value based on position
    pub fn set_variable(&mut self, pos: usize, val: bool) {
        self.variables[pos].is_set = true;
        self.variables[pos].value = val;
    }

    // sets a variable from the variables back to not set based on position
    pub fn unset_variable(&mut self, pos: usize) {
        self.variables[pos].is_set = false;
    }

    // loads the standard cnf benchmark file into the solver
    pub fn load_cnf(&mut self, source: impl Read) -> Result<(), Box<dyn Error>> {
        let buf_reader = BufReader::new(source);
        let mut check = false;
        for maybe_line in buf_reader.lines() {
            let line = maybe_line?;
                    if line.contains("p cnf") {
                        check = true;
                    } else if line.contains("%") {
                        return Ok(());
                    } else if check {
                        let st = line.trim();
                        let st = st.split(" ");
                        let st: Vec<&str> = st.collect();

                        let mut con_group = ConnectionGroup::new();

                        for i in 0..3 {
                            let mut var_name = st[i];
                            let neg = var_name.contains("-");
                            if neg {
                                var_name = &var_name[1..];
                            }

                            let var_pos = self.add_variable(var_name.to_owned()).unwrap();
                            let connection = Connection::new(var_pos, !neg);

                            con_group.connections.push(connection);
                        }

                        self.connections.push(con_group);
                    }
                }
        Ok(())
    }

    // checks an individual connection
    pub fn check_connection(&self, connection: &Connection) -> Option<bool> {
        let var = self.get_variable(connection.var_pos)?;

        if !var.is_set {
            return Some(true);
        }

        Some(var.value == connection.val)
    }

    // checks a connection group
    pub fn check_group(&mut self, group_pos: usize) -> Option<bool> {
        let mut check = false;

        for i in 0..self.connections[group_pos].connections.len() {
            check |= self.check_connection(&self.connections[group_pos].connections[i])?;
            self.connections_checked += 1;

            if check {
                return Some(check);
            }
        }

        Some(check)
    }

    // solves the sat problem
    pub fn solve(&mut self) -> SolveResult {
        // initialize a vector to hold assigned values
        let mut assigned = Vec::new();

        // push the first value into the assigned values
        assigned.push(self.variables[assigned.len()].pos);

        // while we have at least one value to be assigned
        while assigned.len() > 0 {
            // intialize the check to true
            let mut check = true;

            // if everything is assigned then return true
            if assigned.len() >= self.variables.len() {
                return SolveResult {
                    sat: true,
                    connections_checked: self.connections_checked as u64,
                    num_backtracks: self.backtracks as u64
                };
            }

            // gets the variable to assigned
            let cur = self.get_variable(*assigned.last().unwrap()).unwrap();

            // gets the variables position
            let pos = cur.pos;

            // if the variable isn't set, set it to true
            if !cur.is_set {
                self.set_variable(pos, true);
            }
            // otherwise, if set to true, set to false
            else if cur.value {
                self.set_variable(pos, false);
            }

            // loop through connections and perform out checks
            for con in 0..self.connections.len() {
                if !self.check_group(con).unwrap() {
                    check = false;
                    break;
                }
            }

            // if check is true, push the next variable to be assigned
            if check {
                assigned.push(self.variables[assigned.len()].pos);
            }
            // else, if the value was false, go through and backtrack
            else if !self.get_variable(pos).unwrap().value {
                while !self.get_variable(*assigned.last().unwrap()).unwrap().value {
                    self.unset_variable(*assigned.last().unwrap());
                    assigned.pop();
                    self.backtracks += 1;
                }
            }
        }

        // return false if unsat
        SolveResult {
            sat: false,
            connections_checked: self.connections_checked as u64,
            num_backtracks: self.backtracks as u64
        }
    }

    // prints out the variables
    pub fn print_variables(&self) {
        for var in &self.variables {
            println!("{} {}", var.name, var.value);
        }
    }
}
