// import out structs
use crate::connections::{Connection, ConnectionGroup};
use crate::variable::Variable;

use std::cell::RefCell;
use std::error::Error;
// import required imports
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
    variables: Vec<Rc<RefCell<Variable>>>,

    // connections in the solver
    connection_groups: Vec<ConnectionGroup>,
}

impl Solver {
    // create a new solver instance
    pub fn new() -> Self {
        Solver {
            variables: Vec::new(),
            connection_groups: Vec::new(),
        }
    }

    // adds a variable to the solver, if the variable exists then return its position,
    // otherwise add it and return the position
    pub fn add_variable(&mut self, name: String) -> Result<Rc<RefCell<Variable>>, Box<dyn Error>> {
        for variable in self.variables.iter() {
            if variable.borrow().name == name {
                return Ok(Rc::clone(variable));
            }
        }

        let new_var = Rc::new(RefCell::new(Variable::new(name)));

        self.variables.push(Rc::clone(&new_var));

        Ok(new_var)
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
                let line_parts: Vec<&str> = line.trim().split(' ').collect();

                let mut group = ConnectionGroup::default();

                for i in 0..3 {
                    let mut var_name = line_parts[i];
                    let neg = var_name.contains('-');
                    if neg {
                        var_name = &var_name[1..];
                    }

                    let var_pos = self.add_variable(var_name.to_owned()).unwrap();
                    let connection = Connection::new(var_pos, !neg);

                    group.add_connection(connection);
                }

                self.connection_groups.push(group);
            }
        }

        Ok(())
    }

    // solves the sat problem
    pub fn solve(&mut self) -> SolveResult {
        let mut connections_checked = 0;
        let mut num_backtracks = 0;

        // initialize a vector to hold assigned values
        let mut assigned = vec![Rc::clone(&self.variables[0])];

        // while we have at least one value to be assigned
        while !assigned.is_empty() {
            // if everything is assigned then return true
            if assigned.len() >= self.variables.len() {
                return SolveResult {
                    sat: false,
                    connections_checked,
                    num_backtracks,
                };
            }

            // gets the variable to assigned
            let cur_rc = assigned.last().unwrap().clone();
            {
                let mut cur = cur_rc.borrow_mut();

                // if the variable isn't set, set it to true
                // otherwise, if set to true, set to false
                cur.maybe_value = match cur.maybe_value {
                    None => Some(true),
                    Some(true) => Some(false),
                    Some(false) => unreachable!(),
                };
            }

            let check = self.connection_groups.iter().all(|group| {
                let group_check_result = group.check_group();
                connections_checked += group_check_result.connections_checked;
                group_check_result.success
            });

            if check {
                // if check is true, push the next variable to be assigned
                assigned.push(self.variables[assigned.len()].clone());
            } else {
                // else, backtrack
                while matches!(
                    assigned.last().and_then(|last| last.borrow().maybe_value),
                    Some(false)
                ) {
                    let assigned_last = assigned.pop().unwrap();
                    assigned_last.borrow_mut().maybe_value = None;
                    num_backtracks += 1;
                }
            }
        }

        // return false if unsat
        SolveResult {
            sat: false,
            connections_checked,
            num_backtracks,
        }
    }

    // prints out the variables
    pub fn print_variables(&self) {
        for var in &self.variables {
            println!("{} {:?}", var.borrow().name, var.borrow().maybe_value);
        }
    }

    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    pub fn num_connections(&self) -> usize {
        self.connection_groups
            .iter()
            .map(ConnectionGroup::num_connections)
            .sum()
    }
}
