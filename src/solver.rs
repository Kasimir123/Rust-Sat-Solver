// import out structs
use crate::connections::{Connection, ConnectionGroup};
use crate::variable::Variable;

// import required imports
use std::collections::HashMap;
use std::collections::HashSet;
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
    pub is_uc: bool,
    pub literal_sign: Option<bool>,
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

    pub fn get_next_cur(&self, unsat_clauses: &HashSet<usize>) -> CurResult {
        let mut connections_checked = 0;

        let mut set = false;
        let mut min_con: Option<usize> = None;
        let mut min_val: Option<usize> = None;
        let mut literal_sign: Option<bool> = None;
        let mut is_uc: bool = false;

        for i in unsat_clauses {
            let group = self.connection_groups.get(*i).unwrap();
            // let or_check = group.connections.iter().any(|con| {
            //     connections_checked += 1;
            //     self.check_connection_not_null(*con as usize).unwrap()
            // });

            // if !or_check {
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
                min_con = Some(*i);
                min_val = Some(count);
            } else if count < min_val.unwrap() {
                min_con = Some(*i);
                min_val = Some(count);
            }

            if min_val.unwrap() == 1 {
                for connection in group.connections.iter() {
                    if self
                        .variables
                        .get(self.connections.get(*connection).unwrap().var_pos)
                        .unwrap()
                        .value
                        == None
                    {
                        literal_sign = Some(
                            self.connections.get(*connection)
                                .unwrap().val
                        );
                    }
                }
                break;
            }
            // }
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
                    is_uc = min_val.unwrap() == 1;
                    return CurResult {
                        set: true,
                        cur: Some(var.pos),
                        connections_checked,
                        is_uc,
                        literal_sign,
                    };
                }
            }
        }
        return CurResult {
            set: false,
            cur: None,
            connections_checked,
            is_uc,
            literal_sign,
        };
    }

    pub fn get_lcv(&self, cur: &Variable) -> bool {
        let mut connections_checked = 0;
        
        let mut literal_sign: bool;
        let mut var_score = 0;

        for i in 0..self.variable_connections[cur.pos].len() {
            let group_index = self.variable_connections[cur.pos][i];
            let group = self.connection_groups.get(group_index).unwrap();
            let or_check = group.connections.iter().any(|con| {
                connections_checked += 1;
                self.check_connection_not_null(*con as usize).unwrap()
            });

            if !or_check {
                for connection in group.connections.iter() {
                    if self
                        .variables
                        .get(self.connections.get(*connection).unwrap().var_pos)
                        .unwrap()
                        .name
                        == cur.name
                    {
                        literal_sign = self.connections.get(*connection)
                            .unwrap().val;
                        if literal_sign {
                            var_score += 1;
                        } else {
                            var_score -= 1;
                        }
                    }
                }
            }
        }
        if var_score >= 0 {
            return true;
        }
        return false;
    }

    // solves the sat problem
    pub fn solve(&mut self) -> SolveResult {
        // initialize a vector to hold assigned values
        let mut assigned = Vec::new();

        // initializing hash set to keep track of unsatisfied clauses
        let mut unsat_clauses: HashSet<usize> = HashSet::new();
        // initializing vector vector to keep track of clauses
        // satisfied at each assignment
        let mut clauses_sat_at_assignment: Vec<Vec<&usize>> = Vec::new();
        for i in 0..self.variables.len() {
            unsat_clauses.insert(i);
            // the sub-vector of newly satisfied clauses as each assignment
            clauses_sat_at_assignment.push(Vec::new());
        }

        // push the first value into the assigned values
        let next_cur = self.get_next_cur(&unsat_clauses);

        self.connections_checked += next_cur.connections_checked;

        if !next_cur.set {
            return SolveResult {
                sat: true,
                connections_checked: self.connections_checked as u64,
                num_backtracks: self.backtracks as u64,
            };
        }

        assigned.push(next_cur.cur.unwrap());

        // initializing vector to keep track of lcv value assignments
        let mut var_exhausted: Vec<Option<bool>> = Vec::new();
        for i in 00..self.variables.len() {
            var_exhausted.push(None)
        }
        let mut assigned_index = 0;
        let mut lcv_status: Option<bool>;

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

            let mut new_val = None;
            if !next_cur.is_uc {
                // check to see if lcv has been tried
                match var_exhausted.get(assigned_index) {
                    Some(&None) => {
                        lcv_status = Some(false);
                        var_exhausted[assigned_index] = Some(false);
                    },
                    Some(Some(false)) => {
                        lcv_status = Some(true);
                        var_exhausted[assigned_index] = Some(true);
                    },
                    Some(Some(true)) => {
                        lcv_status = None;
                        var_exhausted[assigned_index] = None;
                    },
                    None => unreachable!(),
                }
                new_val = match lcv_status {
                    Some(false) => Some(self.get_lcv(cur)),
                    Some(true) => Some(!cur.value.unwrap()),
                    None => unreachable!(),
                };
            } else {
                new_val = next_cur.literal_sign;
            }
 
            self.variables[pos].value = new_val;

            // loop through connections and perform out checks

            // consider which clauses might be newly satisfied
            // during the following conflict-check
            let mut newly_satisfied_clauses: Vec<&usize> = Vec::new();
            
            let check = self
                .variable_connections
                .get(pos)
                .unwrap()
                .iter()
                .all(|group| {
                    let group_index = group;
                        
                    let group = self.connection_groups.get(*group).unwrap();

                    let or_check = group.connections.iter().any(|con| {
                        connections_checked += 1;
                        self.check_connection(*con as usize).unwrap()
                    });

                    let sat_check = group.connections.iter().any(|con| {
                        connections_checked += 1;
                        self.check_connection_not_null(*con as usize).unwrap()
                    });

                    if sat_check && unsat_clauses.contains(group_index) {
                        newly_satisfied_clauses.push(group_index);
                        unsat_clauses.remove(group_index);
                    }

                    or_check
                });

            // remember which clauses were satisfied at this assignment
            // if the assignment didn't cause a conflict
            if check {
                clauses_sat_at_assignment[assigned_index]
                    = newly_satisfied_clauses;
            }

            self.connections_checked += connections_checked;

            // if check is true, push the next variable to be assigned
            if check {
                let next_cur = self.get_next_cur(&unsat_clauses);

                self.connections_checked += next_cur.connections_checked;

                if !next_cur.set {
                    return SolveResult {
                        sat: true,
                        connections_checked: self.connections_checked as u64,
                        num_backtracks: self.backtracks as u64,
                    };
                }

                assigned.push(next_cur.cur.unwrap());
                assigned_index += 1;
            }
            // else, if the value was false, go through and backtrack
            else {
                for i in 0..clauses_sat_at_assignment[assigned_index].len() {
                    unsat_clauses
                        .insert(*clauses_sat_at_assignment[assigned_index][i]);
                }
                clauses_sat_at_assignment[assigned_index].truncate(0);
                while matches!(
                    var_exhausted.get(assigned_index),
                    Some(Some(true))
                ) {
                    let assigned_last = assigned.pop().unwrap();
                    self.variables[assigned_last].value = None;
                    self.backtracks += 1;
                    var_exhausted[assigned_index] = None;
                    assigned_index -= 1;
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

    pub fn final_check(&mut self) -> bool {
        println!("{} {}", self.connection_groups.len(), self.variables.len());

        for i in 0..self.variables.len() {
            let cur = self.variables.get(i).unwrap();
            let new_val = match cur.value {
                    None => Some(true),
                    Some(true) => Some(true),
                    Some(false) => Some(false),
            };
            self.variables[i].value = new_val;
        }

        let check = self
                .connection_groups
                .iter()
                .all(|group| {

                    let or_check = group.connections.iter().any(|con| {
                        self.check_connection_not_null(*con as usize).unwrap()
                    });

                    or_check
                });
        check
    }

    // prints out the variables
    pub fn print_variables(&self) {
        
        for var in &self.variables {
            println!("{} {}", var.name, var.value.unwrap());
        }
    }
}
