// import out structs
use crate::connections::{Connection, ConnectionGroup};
use crate::variable::Variable;

// import required imports
use std::process;
use std::collections::BTreeSet;
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
        
        if var.value.is_none() {
            return Some(true);
        }

        // let ret = match var.value {
        //     None => Some(true),
        //     _ => Some(false),
        // };

        // if ret? {
        //     return Some(true);
        // }

        Some(var.value? == connection.val)
    }

    // checks an individual connection
    pub fn check_connection_not_null(&self, connection: usize) -> Option<bool> {
        let connection = self.connections.get(connection)?;

        let var = self.variables.get(connection.var_pos)?;

        // if var.value == None {
        //     return Some(false);
        // }
        if var.value.is_none() {
            return Some(false);
        }

        Some(var.value? == connection.val)
    }

    pub fn get_next_cur(&self, unsat_groups: &BTreeSet<usize>) -> CurResult {

        let mut set = false;
        let mut min_con: Option<usize> = None;
        let mut min_val: Option<usize> = None;
        let mut literal_sign: Option<bool> = None;
        let mut is_uc: bool = false;

        // println!("{}", unsat_groups.len());

        for i in unsat_groups.iter() {
            // println!("{}", i);
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
        // process::exit(0x0100);

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
                        connections_checked: 0,
                        is_uc,
                        literal_sign,
                    };
                }
            }
        }
        return CurResult {
            set: false,
            cur: None,
            connections_checked: 0,
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

        let mut unsat_groups: BTreeSet<usize> = BTreeSet::new();
        let mut sat_groups: BTreeSet<usize> = BTreeSet::new();
        let mut groups_sat_at_assignment: Vec<Vec<&usize>> = Vec::new();
        for _i in 0..self.variables.len() {
            groups_sat_at_assignment.push(Vec::new());
        }
        for i in 0..self.connection_groups.len() {
            unsat_groups.insert(i);
        }

        // push the first value into the assigned values
        let next_cur = self.get_next_cur(&unsat_groups);

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
        for _i in 0..self.variables.len() {
            var_exhausted.push(None)
        }
        let mut lcv_status: Option<bool>;

        // while we have at least one value to be assigned
        while !assigned.is_empty() {




            
            // remove after working
            // for group_index in &sat_groups {
            //     let group = self.connection_groups.get(*group_index).unwrap();
            //     let sat_check = group.connections.iter().any(|con| {
            //         self.check_connection_not_null(*con as usize).unwrap()
            //     });
            //     if !sat_check {
            //         for con in self.connection_groups[*group_index].connections.iter() {
            //             let actual_con: &Connection = self.connections.get(*con).unwrap();
            //             // println!("{} {} {}", actual_con.val, self.variables[actual_con.var_pos].name, self.variables[actual_con.var_pos].value.unwrap());
            //         }
            //         println!("top {}", assigned.len() - 1);
            //         // process::exit(1);
            //     }
            // }



            



            // gets the variable to assigned
            let cur = self.variables.get(*assigned.last().unwrap()).unwrap();

            let mut connections_checked = 0;

            // gets the variables position
            let pos = cur.pos;

            let new_val: Option<bool>;
            if !next_cur.is_uc {
                // check to see if lcv has been tried
                match var_exhausted.get(assigned.len() - 1) {
                    Some(&None) => {
                        lcv_status = Some(false);
                        var_exhausted[assigned.len() - 1] = Some(false);
                    },
                    Some(Some(false)) => {
                        lcv_status = Some(true);
                        var_exhausted[assigned.len() - 1] = Some(true);
                    },
                    _ => unreachable!(),
                }
                new_val = match lcv_status {
                    Some(false) => Some(self.get_lcv(cur)),
                    Some(true) => Some(!cur.value.unwrap()),
                    _ => unreachable!(),
                };
            } else {
                var_exhausted[assigned.len() - 1] = Some(true);
                new_val = next_cur.literal_sign;
            }

            // if assigned.len() - 1 == 16 {
            //     match self.variables[pos].value {
            //         None => {
            //             println!("before: None")
            //         },
            //         _ => {
            //             println!("before {}", self.variables[pos].value.unwrap());
            //         }
            //     }
            // }
            self.variables[pos].value = new_val;
            // if assigned.len() - 1 == 16 {
            //     println!("after {}", self.variables[pos].value.unwrap());
            // }

            // reset the possibly satisfied groups for this assignment
            groups_sat_at_assignment[assigned.len() - 1].clear();

            // loop through connections and perform out checks
            
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

                    if sat_check && unsat_groups.contains(group_index) {
                        groups_sat_at_assignment[assigned.len() - 1]
                            .push(group_index);
                    }

                    or_check
                });

            self.connections_checked += connections_checked;





            // remove after working
            // for group_index in &sat_groups {
            //     let group = self.connection_groups.get(*group_index).unwrap();
            //     let sat_check = group.connections.iter().any(|con| {
            //         self.check_connection_not_null(*con as usize).unwrap()
            //     });
            //     if !sat_check {
            //         for con in self.connection_groups[*group_index].connections.iter() {
            //             let actual_con: &Connection = self.connections.get(*con).unwrap();
            //             // println!("{} {} {}", actual_con.val, self.variables[actual_con.var_pos].name, self.variables[actual_con.var_pos].value.unwrap());
            //         }
            //         println!("before {}", assigned.len() - 1);
                    
            //         // if assigned.len() - 1 == 16 {
            //         //     println!("{}", new_val.unwrap());
            //         // }
            //         // process::exit(1);
            //     }
            // }


            

            // if check is true, push the next variable to be assigned
            if check {
                for i in 0..groups_sat_at_assignment[assigned.len() - 1].len() {
                    unsat_groups
                        .remove(groups_sat_at_assignment[assigned.len() - 1][i]);
                    sat_groups
                        .insert(*groups_sat_at_assignment[assigned.len() - 1][i]);
                }




                
                // remove after working
                // for group_index in &sat_groups {
                //     let group = self.connection_groups.get(*group_index).unwrap();
                //     let sat_check = group.connections.iter().any(|con| {
                //         self.check_connection_not_null(*con as usize).unwrap()
                //     });
                //     if !sat_check {
                //         for con in self.connection_groups[*group_index].connections.iter() {
                //             let actual_con: &Connection = self.connections.get(*con).unwrap();
                //             // println!("{} {} {}", actual_con.val, self.variables[actual_con.var_pos].name, self.variables[actual_con.var_pos].value.unwrap());
                //         }
                //         println!("just inserted {}", assigned.len() - 1);
                //         // process::exit(1);
                //     }
                // }

                
                
                let next_cur = self.get_next_cur(&unsat_groups);

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
                    var_exhausted.get(assigned.len() - 1),
                    Some(Some(true))
                        
                ) {


                   
                    // remove after working
                    // for group_index in &sat_groups {
                    //     let group = self.connection_groups.get(*group_index).unwrap();
                    //     let sat_check = group.connections.iter().any(|con| {
                    //         self.check_connection_not_null(*con as usize).unwrap()
                    //     });
                    //     if !sat_check {
                    //         for con in self.connection_groups[*group_index].connections.iter() {
                    //             let actual_con: &Connection = self.connections.get(*con).unwrap();
                    //             // println!("{} {} {}", actual_con.val, self.variables[actual_con.var_pos].name, self.variables[actual_con.var_pos].value.unwrap());
                    //         }
                    //         println!("above {}", assigned.len() - 1);
                    //         println!("{}", group_index);
                    //         // process::exit(1);
                    //     }
                    // }

                    
                    var_exhausted[assigned.len() - 1] = None;

                    // println!("popping: {}", (assigned.len() - 1));
                    // for i in 14..19 {
                    //     println!("printing groups sat for assignment: {}", i);
                    //     for j in 0..groups_sat_at_assignment[i].len() {
                    //         println!("{}", groups_sat_at_assignment[i][j])
                    //     }
                    // }
                
                    for i in 0..groups_sat_at_assignment[assigned.len() - 2].len() {
                        // println!("group index to be switched: {}", groups_sat_at_assignment[assigned.len() - 1][i]);
                        unsat_groups
                            .insert(*groups_sat_at_assignment[assigned.len() - 2][i]);
                        sat_groups
                            .remove(groups_sat_at_assignment[assigned.len() - 2][i]);
                    }

                    let assigned_last = assigned.pop().unwrap();



                    // remove after working
                    // for group_index in &sat_groups {
                    //     let group = self.connection_groups.get(*group_index).unwrap();
                    //     let sat_check = group.connections.iter().any(|con| {
                    //         self.check_connection_not_null(*con as usize).unwrap()
                    //     });
                    //     if !sat_check {
                    //         for con in self.connection_groups[*group_index].connections.iter() {
                    //             let actual_con: &Connection = self.connections.get(*con).unwrap();
                    //             // println!("{} {} {}", actual_con.val, self.variables[actual_con.var_pos].name, self.variables[actual_con.var_pos].value.unwrap());
                    //         }
                    //         println!("below {}", assigned.len() - 1);
                    //         process::exit(1);
                    //     }
                    // }


                    
                    self.variables[assigned_last].value = None;
                    // println!("{}", assigned.len());
                    self.backtracks += 1;
                    // println!("backtracking after exhausting {}", assigned.len());
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
        // println!("{} {}", self.connection_groups.len(), self.variables.len());

        for i in 0..self.variables.len() {
            let cur = self.variables.get(i).unwrap();
            let new_val = match cur.value {
                    None => Some(true),
                    Some(true) => Some(true),
                    Some(false) => Some(false),
            };
            self.variables[i].value = new_val;
        }

        let mut connection_group_index = 0;

        let check = self
                .connection_groups
                .iter()
                .all(|group| {

                    let or_check = group.connections.iter().any(|con| {
                        self.check_connection_not_null(*con as usize).unwrap()
                    });

                    // if !or_check {
                    //     println!("{}", connection_group_index);
                    //     for con in group.connections.iter() {
                    //         let actual_con: &Connection = self.connections.get(*con).unwrap();
                    //         println!("{} {} {}", actual_con.val, self.variables[actual_con.var_pos].name, self.variables[actual_con.var_pos].value.unwrap());
                    //     }
                    // }

                    connection_group_index += 1;

                    or_check
                });
     
        // for con in self.connection_groups[19].connections.iter() {
        //     let actual_con: &Connection = self.connections.get(*con).unwrap();
        //     println!("{} {} {}", actual_con.val, self.variables[actual_con.var_pos].name, self.variables[actual_con.var_pos].value.unwrap());
        // }
        
        check
    }

    // prints out the variables
    pub fn print_variables(&self) {
        
        for var in &self.variables {
            println!("{} {}", var.name, var.value.unwrap());
        }
    }
}
