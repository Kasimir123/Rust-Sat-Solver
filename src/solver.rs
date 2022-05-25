// import out structs
use crate::connections::{Connection, ConnectionGroup};
use crate::variable::Variable;
use crate::conflict_set::ConflictSet;

// import required imports
use std::collections::BTreeSet;
use std::error::Error;
use std::io::{BufRead, BufReader, Read};


// Struct for the response from solve
pub struct SolveResult {
    pub sat: bool,
    pub connections_checked: u64,
    pub num_backtracks: u64,
}

// Struct for the response from get next variable function
pub struct CurResult {
    pub set: bool,
    pub cur: Option<usize>,
    pub connections_checked: usize,
    pub is_uc: bool,
    pub literal_sign: Option<bool>,
}

pub struct CheckResult {
    pub check: bool,
    pub groups_sat: Vec<usize>,
    pub connections_checked: usize,
    pub min_group_index: Option<usize>,
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

        // Loop through the variables to see if the variable exists
        for i in 0..self.variables.len() {

            // if it does then return the index
            if self.variables[i].name.eq(&name) {
                return Some(i);
            }
        }

        // if not, then make a new variable and set the position
        let mut new_var = Variable::new(name);
        new_var.pos = self.variables.len();

        // push the variable to the variables
        self.variables.push(new_var);

        // create a new vector for all connections attached to this variable
        self.variable_connections.push(Vec::new());

        // return the variable position
        Some(self.variables.len() - 1)
    }

    // loads the standard cnf benchmark file into the solver
    pub fn load_cnf(&mut self, source: impl Read) -> Result<(), Box<dyn Error>> {

        // create a reader from the bytes we passed the function
        let buf_reader = BufReader::new(source);

        // check if we can start parsing the contents
        let mut check = false;

        // loop through all the lines
        for maybe_line in buf_reader.lines() {


            // unwrap the line
            let line = maybe_line?;

            // if the line contains p cnf we can start parsing the CNF
            if line.contains("p cnf") {
                check = true;
            } 
            // if % then we can stop parsing the cnf
            else if line.contains('%') {
                return Ok(());
            } 
            // otherwise parse the cnf
            else if check {

                // clean up the line and split by spaces
                let st = line.trim();
                let st = st.split(' ');
                let st: Vec<&str> = st.collect();

                // initialize a connection group
                let mut con_group = ConnectionGroup::default();

                // get the 3 variables
                for i in 0..st.len() {
                    let mut var_name = st[i];

                    if var_name.eq("0") {
                        continue;
                    }

                    // if the variable is negated then set neg to true and remove the -
                    // from the variable name
                    let neg = var_name.contains('-');
                    if neg {
                        var_name = &var_name[1..];
                    }

                    // add the variable and get its position
                    let var_pos = self.add_variable(var_name.to_owned()).unwrap();

                    // create the connection
                    let connection = Connection::new(var_pos, !neg);

                    // push the connection and add the connection to the variables connections
                    self.connections.push(connection);
                    self.variable_connections
                        .get_mut(var_pos)
                        .unwrap()
                        .push(self.connection_groups.len());

                    // add the connection position to the connection group
                    con_group.connections.push(self.connections.len() - 1);
                }

                // push the connection group to the connection groups
                self.connection_groups.push(con_group);
            }
        }
        Ok(())
    }

    // checks an individual connection
    pub fn check_connection(&self, connection: usize) -> Option<bool> {

        // gets the connection based on the position
        let connection = self.connections.get(connection)?;

        // gets the variable in the connection
        let var = self.variables.get(connection.var_pos)?;

        // if var is none then return true
        if var.value.is_none() {
            return Some(true);
        }

        // otherwise return if the variable is equal to the value
        Some(var.value? == connection.val)
    }

    // checks an individual connection and returns false for none
    pub fn check_connection_not_null(&self, connection: usize) -> Option<bool> {

        // gets the connection based on the position
        let connection = self.connections.get(connection)?;

        // gets the variable in the connection
        let var = self.variables.get(connection.var_pos)?;

        // if the variable is none then return false
        if var.value.is_none() {
            return Some(false);
        }

        // otherwise return if the variable is equal to the value
        Some(var.value? == connection.val)
    }

    pub fn get_next_cur(&self, unsat_groups: &BTreeSet<usize>) -> CurResult {
        // println!("num unsat_groups {}", unsat_groups.len());
        let mut set = false;
        let mut min_con: Option<usize> = None;
        let mut min_val: Option<usize> = None;
        let mut literal_sign: Option<bool> = None;
        let mut is_uc: bool = false;

        for i in unsat_groups.iter() {
            // println!("iter");
            let group = self.connection_groups.get(*i).unwrap();

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
                // println!("set");
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
                        literal_sign = Some(self.connections.get(*connection).unwrap().val);
                    }
                }
                break;
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
                // println!("{}", var.value == None);
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
        // println!("test");
        CurResult {
            set: false,
            cur: None,
            connections_checked: 0,
            is_uc,
            literal_sign,
        }
    }

    pub fn get_lcv(&self, cur: &Variable) -> bool {
        let mut connections_checked = 0;

        let mut literal_sign: bool;
        let mut var_score = 0;

        // for group_index in variable_unsat_groups {
        //     let group = self.connection_groups.get(*group_index).unwrap();
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
                        literal_sign = self.connections.get(*connection).unwrap().val;
                        if literal_sign {
                            var_score += 1;
                        } else {
                            var_score -= 1;
                        }
                    }
                }
            }
        }
        var_score >= 0
    }

    pub fn do_check(&self,  unsat_groups: &BTreeSet<usize>, var_assigned_index: &Vec<usize>, pos: &usize) -> CheckResult {
        let mut check = true;
        let mut connections_checked = 0;
        let mut groups_sat: Vec<usize> = Vec::new();
        let mut min_group_index: Option<usize> = None;
        let mut min_var_assiged_indices: Vec<usize> = Vec::new();
        for group_index in self.variable_connections[*pos].iter() {

            let group = self.connection_groups.get(*group_index).unwrap();

            let or_check = group.connections.iter().any(|con| {
                connections_checked += 1;
                self.check_connection(*con as usize).unwrap()
            });

            let sat_check = group.connections.iter().any(|con| {
                connections_checked += 1;
                self.check_connection_not_null(*con as usize).unwrap()
            });

            if sat_check && unsat_groups.contains(&group_index) {
                groups_sat.push(*group_index);
            }

            if !or_check {
                check = false;
                if matches!(min_group_index, None) {
                    let group = self.connection_groups.get(*group_index).unwrap();
                    for con in group.connections.iter() {
                        if self.connections.get(*con).unwrap().var_pos == *pos {continue;}
                        min_group_index = Some(*group_index);
                        min_var_assiged_indices.push(var_assigned_index[self.connections.get(*con).unwrap().var_pos]);
                    }
                } else {
                    let mut pos_var_assiged_indices: Vec<usize> = Vec::new();
                    let group = self.connection_groups.get(*group_index).unwrap();
                    for con in group.connections.iter() {
                        if self.connections.get(*con).unwrap().var_pos == *pos {continue;}
                        pos_var_assiged_indices.push(var_assigned_index[self.connections.get(*con).unwrap().var_pos]);
                    }
                    if pos_var_assiged_indices.iter().max() < min_var_assiged_indices.iter().max() {
                        min_group_index = Some(*group_index);
                        min_var_assiged_indices = pos_var_assiged_indices;
                    }
                }
            }
        };
        CheckResult {
            check,
            groups_sat,
            connections_checked,
            min_group_index,
        }
    }

    // solves the sat problem
    pub fn solve(&mut self) -> SolveResult {
        // initialize a vector to hold assigned values
        let mut assigned = Vec::new();

        let mut unsat_groups: BTreeSet<usize> = BTreeSet::new();
        let mut groups_sat_at_assignment: Vec<Vec<usize>> = Vec::new();
        for _i in 0..self.variables.len() {
            groups_sat_at_assignment.push(Vec::new());
        }
        for i in 0..self.connection_groups.len() {
            unsat_groups.insert(i);
        }

        // let mut variable_unsat_groups: Vec<BTreeSet<usize>> = Vec::new();
        // // let mut refs: Vec<&BTreeSet<usize>> = Vec::new();
        // // let mut dummy: Vec<BTreeSet<usize>> = Vec::new();
        // for i in 0..self.variables.len() {
        //     let mut var_con_set = BTreeSet::new();
        //     let var_cons = self.variable_connections.get(i).unwrap();
        //     for group in var_cons.iter() {
        //         var_con_set.insert(*group);
        //     }
        //     variable_unsat_groups.push(var_con_set);
        // }

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

        let mut conflict_set: ConflictSet = ConflictSet::new();
        // let mut conflict_set: Vec<BTreeSet<usize>> = Vec::new();
        // for _i in 0..self.variables.len() {
        //     conflict_set.push(BTreeSet::new());
        // }

        let mut var_assigned_index: Vec<usize> = Vec::new();
        for _i in 0..self.variables.len() {
            var_assigned_index.push(0);
        }

        // while we have at least one value to be assigned
        while !assigned.is_empty() {
            // println!("{}", assigned.len());
            // gets the variable to assigned
            let cur = self.variables.get(*assigned.last().unwrap()).unwrap();

            // gets the variables position
            let pos = cur.pos;
            
            let new_val: Option<bool>;
            if !next_cur.is_uc {
                // check to see if lcv has been tried
                match var_exhausted.get(assigned.len() - 1) {
                    Some(&None) => {
                        lcv_status = Some(false);
                        var_exhausted[assigned.len() - 1] = Some(false);
                    }
                    Some(Some(false)) => {
                        lcv_status = Some(true);
                        var_exhausted[assigned.len() - 1] = Some(true);
                    }
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

            self.variables[pos].value = new_val;

            // reset the possibly satisfied groups for this assignment
            groups_sat_at_assignment[assigned.len() - 1].clear();

            // loop through connections and perform out checks

            let check_result = self.do_check( &unsat_groups, &var_assigned_index, &pos);
            let check = check_result.check;

            if !check {
                for con in self.connection_groups.get(check_result.min_group_index.unwrap()).unwrap().connections.iter() {
                    let con = self.connections.get(*con).unwrap();
                    if con.var_pos != pos {
                        if !conflict_set.var_set[pos][con.var_pos] {
                            conflict_set.var_list[pos][conflict_set.list_len[pos]] = con.var_pos;
                            conflict_set.list_len[pos] += 1;
                            conflict_set.var_set[pos][con.var_pos] = true;
                        }
                    }
                }
            }

            self.connections_checked += check_result.connections_checked;

            // if check is true, push the next variable to be assigned
            if check {
                for group_sat in check_result.groups_sat.iter() {
                    groups_sat_at_assignment[assigned.len() - 1].push(*group_sat);
                }
                for i in 0..groups_sat_at_assignment[assigned.len() - 1].len() {
                    let group = groups_sat_at_assignment[assigned.len() - 1][i];
                    unsat_groups.remove(&group);
                    // let con_group = self.connection_groups.get(group).unwrap();
                    // for con in con_group.connections.iter() {
                    //     let var = self.connections.get(*con).unwrap().var_pos;
                    //     variable_unsat_groups[var].remove(&group);

                    // }
                }

                let next_cur = self.get_next_cur(&unsat_groups);

                self.connections_checked += next_cur.connections_checked;

                if !next_cur.set {
                    // println!("test");
                    return SolveResult {
                        sat: true,
                        connections_checked: self.connections_checked as u64,
                        num_backtracks: self.backtracks as u64,
                    };
                }

                let next_var_pos = next_cur.cur.unwrap();
                assigned.push(next_var_pos);
                for i in 0..conflict_set.list_len[next_var_pos] {
                    conflict_set.var_set[next_var_pos][conflict_set.var_list[next_var_pos][i]] = false;
                }
                conflict_set.list_len[next_var_pos] = 0;
                // conflict_set[assigned[assigned.len() - 1]].clear();
                var_assigned_index[next_var_pos] = assigned.len() - 1;
            }
            // else, if the value was false, go through and backtrack
            else {
                let mut assignment = assigned[assigned.len() - 1];
                if matches!(var_exhausted.get(assigned.len() - 1), Some(Some(true))) {
                    loop {
                        // println!("{}", assigned.len());
                        // if conflict_set[assignment].contains(&assigned[assigned.len() - 1])
                        if conflict_set.var_set[assignment][assigned[assigned.len() - 1]] {
                            // let mut temp: Vec<usize> = Vec::new();
                            // for conflict in conflict_set[assignment].iter() {
                            //     temp.push(*conflict);
                            // }
                            // for temp_var in temp.iter() {
                            //     conflict_set[assigned[assigned.len() - 1]].insert(*temp_var);
                            // }
                            for i in 0..conflict_set.list_len[assignment] {
                                let move_conflict = conflict_set.var_list[assignment][i];
                                let last_var = assigned[assigned.len() - 1];
                                if !conflict_set.var_set[last_var][move_conflict] {
                                    conflict_set.var_list[last_var][conflict_set.list_len[last_var]] = move_conflict;
                                    conflict_set.list_len[last_var] += 1;
                                    conflict_set.var_set[last_var][move_conflict] = true;
                                }
                            }
                            assignment = assigned[assigned.len() - 1];
                            // do I need to remove assignment from its own conflict set?
                            if matches!(var_exhausted.get(assigned.len() - 1), Some(Some(false))) {
                                break;
                            }
                        }
                        var_exhausted[assigned.len() - 1] = None;
                        for i in 0..groups_sat_at_assignment[assigned.len() - 2].len() {
                            let group = groups_sat_at_assignment[assigned.len() - 2][i];
                            unsat_groups.insert(group);
                            // let con_group = self.connection_groups.get(group).unwrap();
                            // for con in con_group.connections.iter() {
                            //     let var = self.connections.get(*con).unwrap().var_pos;
                            //     variable_unsat_groups[var].insert(group);

                            // }
                        }
                        let to_reset = assigned.pop().unwrap();
                        self.variables[to_reset].value = None;
                        self.backtracks += 1;
                    }
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

        let check = self.connection_groups.iter().all(|group| {
            let or_check = group
                .connections
                .iter()
                .any(|con| self.check_connection_not_null(*con as usize).unwrap());
            connection_group_index += 1;

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
