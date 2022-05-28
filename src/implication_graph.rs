use crate::antecedent::Antecedent;
use crate::connections::{Connection, ConnectionGroup};

use std::collections::BTreeSet;

pub struct ImplicationGraph {
    pub learned: BTreeSet<usize>,
}

impl ImplicationGraph {
    pub fn new(assigned: &Vec<usize>, ant_k: &usize, antecedents: &Vec<Antecedent>, var_assigned_index: &Vec<usize>, groups: &Vec<ConnectionGroup>, connections: &Vec<Connection>) -> Self {
        let assigned_var = assigned[assigned.len() - 1];
        let mut learned: BTreeSet<usize> = BTreeSet::new();
        let mut learned_not_anted: BTreeSet<usize> = BTreeSet::new();
        let d = antecedents[assigned.len() - 1].d;

        for con in groups.get(*ant_k).unwrap().connections.iter() {
            let connection = connections.get(*con).unwrap();
            // if connection.var_pos != assigned_var {
            learned.insert(*con);
            learned_not_anted.insert(*con);
            // }
        }

        while !learned_not_anted.is_empty() {
            let mut max_assigned_index: Option<usize> = None;
            // let mut var_max_assigned_index: Option<usize> = None;
            let mut con_max_assigned_index: Option<usize> = None;
            for con in learned_not_anted.iter() {
                let var = connections.get(*con).unwrap().var_pos;
                let assigned_index = var_assigned_index[var];
                if matches!(max_assigned_index, None) || assigned_index > max_assigned_index.unwrap() {
                    max_assigned_index = Some(assigned_index);
                    // var_max_assigned_index = Some(var);
                    con_max_assigned_index = Some(*con);
                }
                // if matches!(max_assigned_index, None) || assigned_index < max_assigned_index.unwrap() {
                //     max_assigned_index = Some(assigned_index);
                //     // var_max_assigned_index = Some(var);
                //     con_max_assigned_index = Some(*con);
                // }
            }
            learned_not_anted.remove(&con_max_assigned_index.unwrap());
            if !antecedents[max_assigned_index.unwrap()].is_uc {
                continue;
            }
            // resolve lits of groups[ants[max_ass_ind]] w/ learned
            let mut add_to_learned: BTreeSet<usize> = BTreeSet::new();
            let mut remove_from_learned: BTreeSet<usize> = BTreeSet::new();
            for con in groups.get(antecedents.get(max_assigned_index.unwrap()).unwrap().antecedent.unwrap()).unwrap().connections.iter() {
                let connection = connections.get(*con).unwrap();
                let val = connection.val;
                let var_pos = connection.var_pos;
                let mut contains_var: bool = false;
                let mut same_sign: bool = false;
                for con_inner in learned.iter() {
                    let connection_inner = connections.get(*con_inner).unwrap();
                    let var_pos_inner = connection_inner.var_pos;
                    if var_pos == var_pos_inner {
                        contains_var = true;
                        let val_inner = connection_inner.val;
                        if (val as i32) == (val_inner as i32) {
                            same_sign = true;
                        }
                    }
                }
                if !contains_var {
                    add_to_learned.insert(*con);
                } else {
                    if !same_sign {
                        remove_from_learned.insert(*con);
                    }
                }
            }
            for con in add_to_learned.iter() {
                learned.insert(*con);
                learned_not_anted.insert(*con);
            }
            for con in remove_from_learned.iter() {
                learned.remove(con);
                learned_not_anted.remove(con);
            }
            // if only one lit in learned with d = d, break
            let mut count: usize = 0;
            for con in learned.iter() {
                let connection = connections.get(*con).unwrap();
                let var_pos = connection.var_pos;
                let index = var_assigned_index.get(var_pos).unwrap();

                if antecedents.get(*index).unwrap().d == d {
                    count += 1;
                    if count > 1 {                        
                        break;
                    }
                }
            }
            if count > 1 {
                break;
            }
        }

        // add to unsat group and var unsat

        ImplicationGraph {
            learned
        }
    }
}
