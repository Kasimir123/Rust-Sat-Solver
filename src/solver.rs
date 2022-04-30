// import out structs
use crate::connections::{Connection, ConnectionGroup};
use crate::variable::Variable;

// import required imports
use std::collections::HashMap;
use std::error::Error;

use std::io::{BufRead, BufReader, Read};

pub struct SolveResult {
    pub sat: bool,
    pub connections_checked: u64,
    pub num_backtracks: u64,
}

pub struct CurResult {
    pub set: bool,
    pub cur: Option<usize>,
    pub connections_checked: usize,
}

#[derive(Default)]
pub struct Solver {
    // variables in the solver
    variables: Vec<Variable>,

    // connections in the solver
    connection_groups: Vec<ConnectionGroup>,

    connections: Vec<Connection>,

    variable_connections: Vec<Vec<usize>>,

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
            variable_connections: Vec::new(),
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
        self.variable_connections.push(Vec::new());

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
                    self.variable_connections
                        .get_mut(var_pos)
                        .unwrap()
                        .push(self.connection_groups.len());

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

    // checks an individual connection
    pub fn check_connection_not_null(&self, connection: usize) -> Option<bool> {
        let connection = self.connections.get(connection)?;

        let var = self.variables.get(connection.var_pos)?;

        if var.value == None {
            return Some(false);
        }

        Some(var.value? == connection.val)
    }

    pub fn get_next_cur(&self) -> CurResult {
        let mut connections_checked = 0;

        let mut set = false;
        let mut min_con: Option<usize> = None;
        let mut min_val: Option<usize> = None;

        for i in 0..self.connection_groups.len() {
            let group = self.connection_groups.get(i).unwrap();
            let or_check = group.connections.iter().any(|con| {
                connections_checked += 1;
                self.check_connection_not_null(*con as usize).unwrap()
            });

            if !or_check {
                let mut count = 0;
                for connection in group.connections.iter() {
                    if self
                        .variables
                        .get(self.connections.get(*connection).unwrap().var_pos)
                        .unwrap()
                        .value
                        == None
                    {
                        count += 1;
                    }
                }
                if !set {
                    set = true;
                    min_con = Some(i);
                    min_val = Some(count);
                } else if count < min_val.unwrap() {
                    min_con = Some(i);
                    min_val = Some(count);
                }

                if min_val.unwrap() == 1 {
                    break;
                }
            }
        }

        if set {
            for con in self
                .connection_groups
                .get(min_con.unwrap())
                .unwrap()
                .connections
                .iter()
            {
                let var = self
                    .variables
                    .get(self.connections.get(*con).unwrap().var_pos)
                    .unwrap();
                if var.value == None {
                    return CurResult {
                        set: true,
                        cur: Some(var.pos),
                        connections_checked,
                    };
                }
            }
        }
        return CurResult {
            set: false,
            cur: None,
            connections_checked,
        };
    }

    pub fn get_lcv(&self, cur: &Variable) -> bool {
        let mut connections_checked = 0;

        let mut set = false;
        let mut min_con: Option<usize> = None;
        let mut min_val: Option<usize> = None;

        for i in 0..self.connection_groups.len() {
            let group = self.connection_groups.get(i).unwrap();
            let or_check = group.connections.iter().any(|con| {
                connections_checked += 1;
                self.check_connection_not_null(*con as usize).unwrap()
            });

            if !or_check {
                let mut count = 0;
                for connection in group.connections.iter() {
                    if self
                        .variables
                        .get(self.connections.get(*connection).unwrap().var_pos)
                        .unwrap()
                        .value
                        == None
                    {
                        count += 1;
                    }
                }
                if !set {
                    set = true;
                    min_con = Some(i);
                    min_val = Some(count);
                } else if count < min_val.unwrap() {
                    min_con = Some(i);
                    min_val = Some(count);
                }

                if min_val.unwrap() == 1 {
                    break;
                }
            }
        }

        if set {
            for con in self
                .connection_groups
                .get(min_con.unwrap())
                .unwrap()
                .connections
                .iter()
            {
                let var = self
                    .variables
                    .get(self.connections.get(*con).unwrap().var_pos)
                    .unwrap();
                if var.value == None {
                    return CurResult {
                        set: true,
                        cur: Some(var.pos),
                        connections_checked,
                    };
                }
            }
        }
        return true;
    }

    // solves the sat problem
    pub fn solve(&mut self) -> SolveResult {
        // initialize a vector to hold assigned values
        let mut assigned = Vec::new();

        // push the first value into the assigned values
        let next_cur = self.get_next_cur();

        self.connections_checked += next_cur.connections_checked;

        if !next_cur.set {
            return SolveResult {
                sat: true,
                connections_checked: self.connections_checked as u64,
                num_backtracks: self.backtracks as u64,
            };
        }

        assigned.push(next_cur.cur.unwrap());

        let mut cur_value_switch = 1;

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
            let mut cur = self.variables.get(*assigned.last().unwrap()).unwrap();

            let mut connections_checked = 0;

            // gets the variables position
            let pos = cur.pos;
            
            let new_val = match cur_value_switch {
                1 => Some(self.get_lcv(cur)),
                2 => Some(cur.value),
                3 => unreachable!(),
            };

            self.variables[pos].value = new_val;

            // loop through connections and perform out checks

            let check = self
                .variable_connections
                .get(pos)
                .unwrap()
                .iter()
                .all(|group| {
                    let group = self.connection_groups.get(*group).unwrap();

                    let or_check = group.connections.iter().any(|con| {
                        connections_checked += 1;
                        self.check_connection(*con as usize).unwrap()
                    });

                    or_check
                });

            self.connections_checked += connections_checked;

            // if check is true, push the next variable to be assigned
            if check {
                let next_cur = self.get_next_cur();

                self.connections_checked += next_cur.connections_checked;

                if !next_cur.set {
                    return SolveResult {
                        sat: true,
                        connections_checked: self.connections_checked as u64,
                        num_backtracks: self.backtracks as u64,
                    };
                }

                assigned.push(next_cur.cur.unwrap());
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
