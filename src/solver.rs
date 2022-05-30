// import out structs
use crate::connections::{Connection, ConnectionGroup};
use crate::variable::Variable;
use crate::conflict_set::ConflictSet;
use crate::sat_linked_hash_set::SatLinkedHashSet;
use crate::implication_graph::ImplicationGraph;
use crate::antecedent::Antecedent;
// use crate::learned_clause::LearnedClause;

// import required imports
use std::collections::{BTreeSet, VecDeque};
use std::error::Error;
use std::io::{BufRead, BufReader, Read};

// use deepmesa::lists::linkedlist::Node;
// use  deepmesa::lists::LinkedList;

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
    pub antecedent: Option<usize>,
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
    // create a new solver instances
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
    pub fn get_var_max_deg(&self, variable_unsat_groups: &SatLinkedHashSet, min_groups_vars: &BTreeSet<usize>) -> usize{
        let mut max_deg = 0;
        let mut var_max_deg = usize::MAX;
        for var_pos in min_groups_vars.iter() {
            let deg = 400000 - variable_unsat_groups.open_spots_len[*var_pos];
            // println!("deg: {}", deg);
            if deg > max_deg {
                var_max_deg = *var_pos;
                max_deg = deg;
            }
        }
        var_max_deg
    }
    pub fn get_next_cur(&self, unsat_groups: &BTreeSet<usize>, variable_unsat_groups: &SatLinkedHashSet) -> CurResult {
        // println!("num unsat_groups {}", unsat_groups.len());
        let mut set = false;
        let mut min_groups: Vec<usize> = Vec::new();
        let mut min_groups_vars: BTreeSet<usize> = BTreeSet::new();
        let mut min_val: Option<usize> = None;
        let mut literal_sign: Option<bool> = None;
        let mut is_uc: bool = false;
        let mut antecedent: Option<usize> = None;

        for group_index in unsat_groups.iter() {
            // println!("iter");
            let group = self.connection_groups.get(*group_index).unwrap();

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
                min_groups.push(*group_index);
                min_val = Some(count);
            } else if count < min_val.unwrap() {
                min_groups.clear();
                min_groups.push(*group_index);
                min_val = Some(count);
            } else if count == min_val.unwrap() {
                min_groups.push(*group_index);
            }

            if min_val.unwrap() == 1 {
                is_uc = true;
                for connection in group.connections.iter() {
                    let var = self.connections.get(*connection).unwrap().var_pos;
                    if self
                        .variables
                        .get(var)
                        .unwrap()
                        .value
                        == None
                    {
                        literal_sign = Some(self.connections.get(*connection).unwrap().val);
                        antecedent = Some(*group_index);
                    }
                }
                break;
            }
        }

        for group_index in min_groups.iter() {
            let group = self.connection_groups.get(*group_index).unwrap();
            // println!("group_index: {}", group_index);
            // let mut test_count = 0;
            for con in group.connections.iter() {
                let var = self.connections.get(*con).unwrap().var_pos;
                if self.variables.get(var).unwrap().value == None {
                    min_groups_vars.insert(var);
                    
                }
                // else {
                //     test_count += 1;
                // }
                // if test_count > 1 && *group_index == 107 {
                //     println!("yes");
                //     std::process::exit(1);
                // }
            }
        }

        // println!("min_groups.len(): {}", min_groups.len());
        // println!("min_groups_vars.len(): {}", min_groups_vars.len());
        let var_max_deg = self.get_var_max_deg(variable_unsat_groups, &min_groups_vars);

        if set {
            return CurResult {
                set: true,
                cur: Some(var_max_deg),
                connections_checked: 0,
                is_uc,
                literal_sign,
                antecedent,
            }
        }

        // if set {
        //     for con in self
        //         .connection_groups
        //         .get(min_con.unwrap())
        //         .unwrap()
        //         .connections
        //         .iter()
        //     {
        //         let var = self
        //             .variables
        //             .get(self.connections.get(*con).unwrap().var_pos)
        //             .unwrap();
        //         // println!("not assigned: {}", var.value == None);
        //         // println!("{}", min_val.unwrap());
        //         // println!("sat: {}", self.check_connection_not_null(*con).unwrap());
        //         if var.value == None {
        //             // println!("return");
        //             is_uc = min_val.unwrap() == 1;
        //             return CurResult {
        //                 set: true,
        //                 cur: Some(var.pos),
        //                 connections_checked: 0,
        //                 is_uc,
        //                 literal_sign,
        //             };
        //         }
        //     }
        // }
        // println!("test");
        CurResult {
            set: false,
            cur: None,
            connections_checked: 0,
            is_uc,
            literal_sign,
            antecedent,
        }
    }

    pub fn get_lcv(&self, cur: &Variable, variable_unsat_groups: &SatLinkedHashSet) -> bool {
    // pub fn get_lcv(&self, cur: &Variable) -> bool {
        let mut connections_checked = 0;

        let mut literal_sign: bool;
        let mut var_score = 0;

        let pos = cur.pos;
        let mut sat_linked_list_index = variable_unsat_groups.heads[pos];
        while sat_linked_list_index != usize::MAX {
            let group_index = variable_unsat_groups.var_lists[pos][sat_linked_list_index].val;
            let group = self.connection_groups.get(group_index).unwrap();
            sat_linked_list_index = variable_unsat_groups.var_lists[pos][sat_linked_list_index].next;
        // for group_index in variable_unsat_groups.iter() {
        //     let group = self.connection_groups.get(*group_index).unwrap();
        // for i in 0..self.variable_connections[cur.pos].len() {
        //     let group_index = self.variable_connections[cur.pos][i];
        //     let group = self.connection_groups.get(group_index).unwrap();
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

    pub fn do_check(&self,  unsat_groups: &BTreeSet<usize>, var_assigned_index: &Vec<usize>, pos: &usize, variable_unsat_groups: &SatLinkedHashSet) -> CheckResult {
        // pub fn do_check(&self,  unsat_groups: &BTreeSet<usize>, var_assigned_index: &Vec<usize>, pos: &usize) -> CheckResult {
        // println!("pos: {}", *pos);
        // println!("open spots len: {}", variable_unsat_groups.open_spots_len[*pos]);
        let mut check = true;
        let mut connections_checked = 0;
        let mut groups_sat: Vec<usize> = Vec::new();
        let mut min_group_index: Option<usize> = None;
        let mut min_var_assiged_indices: Vec<usize> = Vec::new();
        let mut sat_linked_list_index = variable_unsat_groups.heads[*pos];
        while sat_linked_list_index != usize::MAX {
            let group_index = &variable_unsat_groups.var_lists[*pos][sat_linked_list_index].val;
            let group = self.connection_groups.get(*group_index).unwrap();
            sat_linked_list_index = variable_unsat_groups.var_lists[*pos][sat_linked_list_index].next;
        // for group_index in self.variable_connections.get(*pos).unwrap().iter() {
        //     let group = self.connection_groups.get(*group_index).unwrap();
        // for group_index in variable_unsat_groups.iter() {
        //     let group = self.connection_groups.get(*group_index).unwrap();

            let or_check = group.connections.iter().any(|con| {
                connections_checked += 1;
                self.check_connection(*con as usize).unwrap()
            });

            let sat_check = group.connections.iter().any(|con| {
                connections_checked += 1;
                self.check_connection_not_null(*con as usize).unwrap()
            });

            if sat_check && unsat_groups.contains(group_index) {
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

    // pub fn delete_groups(&self, learned_clauses: &BTreeSet<LearnedClause>) -> Vec<LearnedClause> {
    //     let mut clauses_removed = 
    //     clauses_removed
    // }

    // solves the sat problem
    pub fn solve(&mut self) -> SolveResult {
        let mut cur_lbd: f64;
        let mut s_t_a_q_lbd: VecDeque<f64> = VecDeque::new();
        let mut s_t_a_lbd: f64 = 0.0;
        let mut cum_lbd: f64 = 0.0;
        let mut cur_a: f64 = 0.0; // should be an option
        let mut s_t_a_q_a: VecDeque<f64> = VecDeque::new();
        let mut s_t_a_a: f64 = 0.0;
        let mut tot_conflicts: f64 = 0.0;

        // need to check what sort() really does to con vec
        let mut unique_clauses: BTreeSet<Vec<Connection>> = BTreeSet::new();
        for group in self.connection_groups.iter() {
            let mut lits = Vec::new();
            for con in group.connections.iter() {
                lits.push(*self.connections.get(*con).unwrap());
            }
            lits.sort();
            // let clone = lits.clone();
            // unique_clauses.insert(clone);
            unique_clauses.insert(lits);
        }

        // let mut used_conflict_set = 0;
        // let mut  debug_465 = 0;

        // println!("num con_groups var 5: {}", self.variable_connections[5].len());
        
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

        let mut variable_unsat_groups: SatLinkedHashSet = SatLinkedHashSet::new(&self.variable_connections);

        let mut antecedents: Vec<Antecedent> = Vec::new();
        for _i in 0..self.variables.len() {
            antecedents.push(Antecedent::new());
        }

        // push the first value into the assigned values
        let next_cur = self.get_next_cur(&unsat_groups, &variable_unsat_groups);
        


        self.connections_checked += next_cur.connections_checked;

        if !next_cur.set {
            return SolveResult {
                sat: true,
                connections_checked: self.connections_checked as u64,
                num_backtracks: self.backtracks as u64,
            };
        }

        assigned.push(next_cur.cur.unwrap());

        let mut d: usize = 0;
        antecedents[assigned.len() - 1].is_uc = next_cur.is_uc;
        antecedents[assigned.len() - 1].antecedent = next_cur.antecedent;
        antecedents[assigned.len() - 1].d = d;

        // initializing vector to keep track of lcv value assignments
        let mut var_exhausted: Vec<Option<bool>> = Vec::new();
        for _i in 0..self.variables.len() {
            var_exhausted.push(None)
        }
        let mut lcv_status: Option<bool>;

        let mut conflict_set: ConflictSet = ConflictSet::new();

        let mut var_assigned_index: Vec<usize> = Vec::new();
        for _i in 0..self.variables.len() {
            var_assigned_index.push(0);            
        }

        let mut prev_restart: f64 = 0.0;

        // while we have at least one value to be assigned
        while !assigned.is_empty() {
            
            // debug_465 += 1;
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
                    Some(false) => Some(self.get_lcv(cur, &variable_unsat_groups)),
                    // Some(false) => Some(self.get_lcv(cur)),
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
            let check_result = self.do_check(&unsat_groups, &var_assigned_index, &pos, &variable_unsat_groups);
            // let check_result = self.do_check(&unsat_groups, &var_assigned_index, &pos);
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
                    let con_group = self.connection_groups.get(group).unwrap();
                    for con in con_group.connections.iter() {
                        let var = self.connections.get(*con).unwrap().var_pos;
                        // if var == 5 {
                        //     println!("REMOVING pos: {}, assigned.len(): {}, group: {}", pos, assigned.len(), group);
                        //     let mut len_5 = 1;
                        //     let mut head_5 = &variable_unsat_groups.var_lists[5][variable_unsat_groups.heads[5]];
                        //     while head_5.next != usize::MAX {
                        //         len_5 += 1;
                        //         head_5 = &variable_unsat_groups.var_lists[5][head_5.next];
                        //     }
                        //     println!("len_5: {}", len_5);
                        //     if len_5 > 16 {
                        //         std::process::exit(1);
                        //     }
                        // }
                        if variable_unsat_groups.contains(var, group) {
                            variable_unsat_groups.remove(var, group);
                        }
                        // let mut var_unsat_group = variable_unsat_groups.var_sets[var][group];
                        // let node = var_unsat_group.node.unwrap();
                        // var_unsat_group.is_unsat = false;
                        // variable_unsat_groups.var_lists[var].pop_node(&node);
                    }
                }

                // println!("pos: {}", pos);
                let next_cur = self.get_next_cur(&unsat_groups, &variable_unsat_groups);
                // println!("{}", next_cur.set);

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

                if next_cur.is_uc {
                    if assigned.len() > 1 {
                        d = antecedents[assigned.len() - 2].d;
                    }
                } else {
                    if assigned.len() > 1 {
                        d = antecedents[assigned.len() - 2].d + 1;
                    }
                }
                antecedents[assigned.len() - 1].is_uc = next_cur.is_uc;
                antecedents[assigned.len() - 1].antecedent = next_cur.antecedent;
                antecedents[assigned.len() - 1].d = d;

                for i in 0..conflict_set.list_len[next_var_pos] {
                    let var_pos = conflict_set.var_list[next_var_pos][i];
                    conflict_set.var_set[next_var_pos][var_pos] = false;
                }
                conflict_set.list_len[next_var_pos] = 0;
                // conflict_set[assigned[assigned.len() - 1]].clear();
                var_assigned_index[next_var_pos] = assigned.len() - 1;
            }

            // need a case in the following while to deal with exhausted first variable (unsatisfiable)

            // else, if the value was false, go through and backtrack (but first learn a clause)
            else {
                tot_conflicts += 1.0;
                cur_a = (assigned.len() - 1) as f64;
                // using self.backtracks instead of tot_conflicts for restart measure
                //  200 and 2 (50 min conflict) is fast for 175
                // also 500 and 2 (50 min) for 175
                // 2500 and 12 is decent for 175 (considering the high numbers are often awful)
                //
                if s_t_a_q_a.len() < 5000 {
                    s_t_a_a = s_t_a_a + (cur_a - s_t_a_a) / tot_conflicts;
                } else {
                    s_t_a_a = s_t_a_a + cur_a / 5000.0 - s_t_a_q_a.pop_front().unwrap() / 5000.0;
                }
                s_t_a_q_a.push_back(cur_a);

                // clause learning
                // gonna start using check_res.min_g_ind as ant(k)
                let implied: ImplicationGraph = ImplicationGraph::new(
                    &assigned,
                    &check_result.min_group_index.unwrap(),
                    &antecedents,
                    &var_assigned_index,
                    &self.connection_groups,
                    &self.connections,
                );
                let mut learned_clause = ConnectionGroup::new();
                let learned_clause_index = self.connection_groups.len();
                // if learned_clause_index == 107 {
                //     println!("num lits in learned clause: {}", implied.learned.len());
                //     std::process::exit(1);
                // }
                let hack_loop_test = false;

                let mut num_d: BTreeSet<usize> = BTreeSet::new();
                for con in implied.learned.iter() {
                    let connection = self.connections.get(*con).unwrap();
                    let var_pos = connection.var_pos;
                    let var_index = var_assigned_index[var_pos];
                    let var_d = antecedents[var_index].d;
                    num_d.insert(var_d);
                }
                cur_lbd = num_d.len() as f64;
                cum_lbd = cum_lbd + (cur_lbd - cum_lbd) / tot_conflicts;
                if s_t_a_q_lbd.len() < 50 {
                    s_t_a_lbd = cum_lbd;
                } else {
                    s_t_a_lbd = s_t_a_lbd + cur_lbd / 50.0 - s_t_a_q_lbd.pop_front().unwrap() / 50.0;
                }
                s_t_a_q_lbd.push_back(cur_lbd);
                // if num_d.len() < 3 {
                if true {
                    if num_d.len() < 3 {
                        let mut lits = Vec::new();
                        for con in implied.learned.iter() {
                            let connection = *self.connections.get(*con).unwrap();
                            // let con_clone = connection.clone();
                            // lits.push(con_clone);
                            lits.push(connection);
                        }
                        lits.sort();
                        if !unique_clauses.contains(&lits) {
                            unique_clauses.insert(lits);
                            for con in implied.learned.iter() {
                                let connection = self.connections.get(*con).unwrap();
                                let var_pos = connection.var_pos;
                                variable_unsat_groups.insert(var_pos, learned_clause_index);
                                self.variable_connections[var_pos].push(learned_clause_index);
                                let learned_lit = Connection::new(var_pos, connection.val);
                                let learned_lit_index = self.connections.len();
                                self.connections.push(learned_lit);
                                learned_clause.connections.push(learned_lit_index);
                            }
                            self.connection_groups.push(learned_clause);
                            unsat_groups.insert(learned_clause_index);
                        }
                    }
                    // // if learned_clause_index == 107 {
                    // if debug_465 > 81 {
                    //     let test_learned = learned_clause.clone();
                    //     // println!("DEBUG 465: {}", debug_465);
                    //     println!("num lits: {}", test_learned.connections.len());
                    //     let conflict_var = assigned[assigned.len() - 1];
                    //     println!("var of conflict is: {}", conflict_var);
                    //     println!("assigned index is: {}", var_assigned_index[conflict_var]);
                    //     let mut test_check_689 = 0;
                    //     for con in test_learned.connections.iter() {
                    //         let var = self.connections.get(*con).unwrap().var_pos;
                    //         println!("var of lit is: {}", var);
                    //         let var_ass_ind = var_assigned_index[var];
                    //         println!("assigned index is: {}", var_ass_ind );
                    //         println!("is_unit: {}", antecedents[var_ass_ind].is_uc);
                    //         println!("is_exhausted: {}", var_exhausted[var_ass_ind].unwrap());
                    //         if !self.check_connection(*con).unwrap() {
                    //             test_check_689 += 1
                    //         }
                    //         if self.variables.get(var).unwrap().value == None {
                    //             println!("clause is unfinished because var {}", var);
                    //             std::process::exit(1);
                    //         }
                    //     }
                    //     println!("test_check_689 {}", test_check_689);
                    //     std::process::exit(1);
                    // }

                    // learned_clause_inds.push(LearnedClause::new(num_d.len(), implied.learned.len(), learned_clause_index));
                    // learned_clauses.insert(learned_clause_inds.len() - 1);

                    // // aparrently the learned clause always includes the most recently assigned variable so commenting out
                    // let new_clause = &self.connection_groups[self.connection_groups.len() - 1];
                    // let mut new_clause_vars: Vec<usize> = Vec::new();
                    // for con in new_clause.connections.iter() {
                    //     let var = self.connections.get(*con).unwrap().var_pos;
                    //     // if !antecedents[var_assigned_index[var]].is_uc {
                    //     new_clause_vars.push(var);
                    //     // }
                    // }
                    // let mut new_clause_var_indices:  Vec<usize> = Vec::new();
                    // for var_pos in new_clause_vars.iter() {
                    //     new_clause_var_indices.push(var_assigned_index[*var_pos]);
                    // }
                    // // println!("{}", new_clause_var_indices.len());
                    // let go_back_to = *new_clause_var_indices.iter().max().unwrap();
                    // let num_go_back = assigned.len() - 1 - go_back_to;
                    // // this hack loop is backing up further if all of the newly-learned clauses's lits are still assigned
                    // // println!("num go back: {}", num_go_back);
                    // // std::process::exit(1);
                    // for _i in 0..num_go_back {
                    //     println!("test");
                    //     // println!("465 debug: {}", debug_465);
                    //     hack_loop_test = true;
                    //     var_exhausted[assigned.len() - 1] = None;
                    //     for i in 0..groups_sat_at_assignment[assigned.len() - 2].len() {
                    //         let group = groups_sat_at_assignment[assigned.len() - 2][i];
                    //         unsat_groups.insert(group);
                    //         let con_group = self.connection_groups.get(group).unwrap();
                    //         for con in con_group.connections.iter() {
                    //             let var = self.connections.get(*con).unwrap().var_pos;
                    //             if !variable_unsat_groups.contains(var, group) {
                    //                 variable_unsat_groups.insert(var, group);
                    //             }
                    //         }
                    //     }
                    //     let to_reset = assigned.pop().unwrap();
                    //     self.variables[to_reset].value = None;
                    //     self.backtracks += 1;
                    // }
                }
                
                if !hack_loop_test {
                    // println!("{}", debug_465);
                    // std::process::exit(1);
                    let mut assignment = assigned[assigned.len() - 1];
                    // println!("assignment: {}", assignment);
                    if matches!(var_exhausted.get(assigned.len() - 1), Some(Some(true))) {
                        loop {
                            if conflict_set.var_set[assignment][assigned[assigned.len() - 1]] {
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
                                let con_group = self.connection_groups.get(group).unwrap();
                                for con in con_group.connections.iter() {
                                    let var = self.connections.get(*con).unwrap().var_pos;
                                    // if var == 5 {
                                    //     println!("ADDING pos: {}, assigned.len(): {}, group: {}", pos, assigned.len(), group);
                                    //     let mut len_5 = 1;
                                    //     let mut head_5 = &variable_unsat_groups.var_lists[5][variable_unsat_groups.heads[5]];
                                    //     while head_5.next != usize::MAX {
                                    //         len_5 += 1;
                                    //         head_5 = &variable_unsat_groups.var_lists[5][head_5.next];
                                    //     }
                                    //     println!("len_5: {}", len_5);
                                    //     if len_5 > 16 {
                                    //         std::process::exit(1);
                                    //     }
                                    // }
                                    // I think I can remove this condition, but leaving it for safety until everything else works
                                    if !variable_unsat_groups.contains(var, group) {
                                        variable_unsat_groups.insert(var, group);
                                    }
                                    // let node: Node<usize> = variable_unsat_groups.var_lists[var].push_head(group);
                                    // variable_unsat_groups.var_sets[var][group].node = Some(node);
                                    // variable_unsat_groups.var_sets[var][group].is_unsat = true;
                                }
                            }
                            let to_reset = assigned.pop().unwrap();
                            self.variables[to_reset].value = None;
                            self.backtracks += 1;
                            // used_conflict_set += 1;
                        }
                    }
                }
            }
            // let percent_used = ((used_conflict_set as f64) / ((self.backtracks + 1) as f64)) * (100 as f64);
            // println!("{}", percent_used);
            // 50 is min restart
            if tot_conflicts > prev_restart + 50.0 {
                if s_t_a_lbd / cum_lbd > 1.25 && !(cur_a > s_t_a_a) {
                    prev_restart = tot_conflicts;
                    let num_pop = assigned.len() - 1;
                    for _i in 0..num_pop {
                        var_exhausted[assigned.len() - 1] = None;
                        for i in 0..groups_sat_at_assignment[assigned.len() - 2].len() {
                            let group = groups_sat_at_assignment[assigned.len() - 2][i];
                            unsat_groups.insert(group);
                            let con_group = self.connection_groups.get(group).unwrap();
                            for con in con_group.connections.iter() {
                                let var = self.connections.get(*con).unwrap().var_pos;
                                if !variable_unsat_groups.contains(var, group) {
                                    variable_unsat_groups.insert(var, group);
                                }
                            }
                        }
                        let to_reset = assigned.pop().unwrap();
                        self.variables[to_reset].value = None;
                        // self.backtracks += 1;
                    }
                    let to_reset = assigned.pop().unwrap();
                    self.variables[to_reset].value = None;
                    let next_var_pos = next_cur.cur.unwrap();
                    assigned.push(next_var_pos);
                    var_exhausted[assigned.len() - 1] = None;
                    antecedents[assigned.len() - 1].is_uc = next_cur.is_uc;
                    antecedents[assigned.len() - 1].antecedent = next_cur.antecedent;
                    antecedents[assigned.len() - 1].d = 0;
                    for i in 0..conflict_set.list_len[next_var_pos] {
                        let var_pos = conflict_set.var_list[next_var_pos][i];
                        conflict_set.var_set[next_var_pos][var_pos] = false;
                    }
                    conflict_set.list_len[next_var_pos] = 0;
                    var_assigned_index[next_var_pos] = assigned.len() - 1;

                    // // trim clauses
                    // // this doesn't have to be done every restart
                    // let mut clauses_to_remove = Vec::new();
                    // for learned_clause in learned_clauses.iter() {
                    //     let the_index = &learned_clause_inds[*learned_clause].index;
                    //     let the_clause = &learned_clause_inds[*learned_clause];
                    //     // if !(the_clause.lbd < 3 || (the_clause.lbd == 3 && the_clause.lits < 5) || (the_clause.lbd == 4 && the_clause.lits == 4)) {
                    //     // if !((the_clause.lbd < 3) || (the_clause.lbd == 3 && the_clause.lits < 5)) {
                    //     if !(the_clause.lbd < 3) {
                    //         clauses_to_remove.push(*the_index);
                    //     }
                    // }
                    // for group_index in clauses_to_remove.iter() {
                    //     let group = self.connection_groups.get(*group_index).unwrap();
                    //     unsat_groups.remove(&group_index);
                    //     for con in group.connections.iter() {
                    //         let connection = self.connections.get(*con).unwrap();
                    //         let var = connection.var_pos;
                    //         variable_unsat_groups.remove(var, *group_index);
                    //     }
                    //     // learned_clauses.remove(group_index);
                    // }
                    // learned_clauses.clear();
                    // learned_clause_inds.clear();
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
