use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
#[repr(usize)]
pub enum Operator {
    AND,
    OR,
    IMPLIES,
    IIF,
    NONE,
}

#[derive(Clone)]
pub struct PropositionalConnection {
    pub operator: Operator,
    pub is_negated: bool,
    pub variable: Option<String>,
    pub variables: Vec<PropositionalConnection>,
}

impl PropositionalConnection {
    pub fn new(operator: Operator, is_negated: bool, variable: Option<String>) -> Self {
        PropositionalConnection {
            operator,
            is_negated,
            variable,
            variables: Vec::new(),
        }
    }

    pub fn print_string(&self) -> String {
        let mut output = String::new();

        if self.is_negated {
            output.push('¬');
        }

        if self.variable != None {
            output.push_str(&(self.variable.clone().unwrap().to_owned()));
        } else {
            if self.variables.len() > 1 {
                output.push('(');
            }
            for var in 0..self.variables.len() {
                output.push_str(&self.variables[var].print_string());

                if var != self.variables.len() - 1 {
                    match self.operator {
                        Operator::AND => {
                            output.push_str(" ∧ ");
                        }

                        Operator::OR => {
                            output.push_str(" ∨ ");
                        }

                        Operator::IMPLIES => {
                            unimplemented!();
                        }

                        Operator::IIF => {
                            unimplemented!();
                        }

                        _ => {}
                    }
                }
            }
            if self.variables.len() > 1 {
                output.push(')');
            }
        }

        output
    }

    // applies demorgans law to the clause
    fn demorgans(&mut self) -> bool {
        
        // check if we need to apply demorgans law
        let apply_de_morgan = self.variables.len() > 1 && self.is_negated;

        // get the check for if something changed
        let mut check = apply_de_morgan;

        // apply demorgans law to the current clause
        if apply_de_morgan {

            // set to not negated
            self.is_negated = false;

            // change the operator
            self.operator = if self.operator == Operator::AND {
                Operator::OR
            } else {
                Operator::AND
            };
        }

        // loop through all the clauses in the clause
        for var in self.variables.iter_mut() {

            // negate them if we applied demorgans law
            if apply_de_morgan {
                var.is_negated = !var.is_negated;
            }
            check |= var.demorgans();
        }

        return check;
    }

    // performs distributive law on the clause
    fn distributive(&mut self) -> bool {

        // check to see if anything was changed
        let mut check = false;

        // if we are not a variable
        if self.variable == None {

            // if this is an or group
            if self.operator == Operator::OR {

                // Create a group to store the left hand side 
                let mut left_hand_group = PropositionalConnection::new(Operator::NONE, false, None);

                // push the first variable / clause
                left_hand_group.variables.push(self.variables[0].clone());

                // loop througb the rest
                for i in 1..self.variables.len() {
                    let cur_var = &self.variables[i];

                    // if not and then we push it to left hand side
                    if cur_var.operator == Operator::OR || cur_var.operator == Operator::NONE {
                        left_hand_group.variables.push(cur_var.clone());
                    } 
                    // otherwise we perform distributive law
                    else {
                        check = true;

                        // create a new and group
                        let mut new_and = PropositionalConnection::new(Operator::AND, false, None);

                        // loop through the variables for the and clause
                        for var in self.variables[i].variables.iter() {

                            // create a new or group
                            let mut new_or =
                                PropositionalConnection::new(Operator::OR, false, None);

                            // copy left hand side variables into the group
                            for left_var in left_hand_group.variables.iter() {
                                new_or.variables.push(left_var.clone());
                            }

                            // copy the and group variable into the or group
                            new_or.variables.push(var.clone());

                            // sort so ands are always on the right side
                            new_or.variables.sort_by(|a, b| {
                                if a.operator == b.operator {
                                    return std::cmp::Ordering::Equal;
                                } else if a.operator == Operator::AND {
                                    return std::cmp::Ordering::Greater;
                                } else {
                                    return std::cmp::Ordering::Less;
                                }
                            });

                            // push the new or group to the and group
                            new_and.variables.push(new_or);
                        }

                        left_hand_group.variables.clear();

                        // sort so ands are always on the right side
                        new_and.variables.sort_by(|a, b| {
                            if a.operator == b.operator {
                                return std::cmp::Ordering::Equal;
                            } else if a.operator == Operator::AND {
                                return std::cmp::Ordering::Greater;
                            } else {
                                return std::cmp::Ordering::Less;
                            }
                        });

                        // add the and group to the left hand group
                        left_hand_group.variables.push(new_and);
                    }
                }

                self.variables.clear();

                // sort so ands are always on the right side
                left_hand_group.variables.sort_by(|a, b| {
                    if a.operator == b.operator {
                        return std::cmp::Ordering::Equal;
                    } else if a.operator == Operator::AND {
                        return std::cmp::Ordering::Greater;
                    } else {
                        return std::cmp::Ordering::Less;
                    }
                });

                // copy the items from the left hand group into the current variable
                if left_hand_group.variables.len() == 1 {
                    self.operator = left_hand_group.variables[0].operator.clone();
                    for left_var in left_hand_group.variables[0].variables.iter() {
                        self.variables.push(left_var.clone());
                    }
                } else {
                    for left_var in left_hand_group.variables.iter() {
                        self.variables.push(left_var.clone());
                    }
                }
            }

            // check the other clauses
            for var in self.variables.iter_mut() {
                check |= var.distributive();
            }
        }

        return check;
    }

    // cleans up the cnf structure
    fn clean_cnf(&mut self) -> bool {
        // check to see if any changes were made
        let mut check = false;

        // gets the variables and prepares it for the new ones
        let variables = self.variables.clone();
        self.variables.clear();

        // if we are in an and group
        if self.operator == Operator::AND {
            for var in variables.iter() {
                // if one of the sub groups is also an and group
                if var.operator == Operator::AND {
                    check = true;

                    // then bring all items into the upper group
                    for new_var in var.variables.iter() {
                        self.variables.push(new_var.clone());
                    }
                } else {
                    self.variables.push(var.clone());
                }
            }
        }
        // if we are in an or group
        else if self.operator == Operator::OR {
            for var in variables.iter() {
                // if one of the sub groups is also an or group
                if var.operator == Operator::OR {
                    check = true;

                    // then bring all items into the upper group
                    for new_var in var.variables.iter() {
                        self.variables.push(new_var.clone());
                    }
                } else {
                    self.variables.push(var.clone());
                }
            }
        }
        // otherwise add the variables back in
        else {
            for var in variables.iter() {
                self.variables.push(var.clone());
            }
        }

        // removes clauses if we have a variable negated and not negated in an or group: (a or ~a)
        let mut removable = Vec::new();
        for (i, variable) in self.variables.iter_mut().enumerate() {
            if variable.operator == Operator::OR {
                let mut variables: HashMap<String, bool> = HashMap::new();
                for var in variable.variables.iter() {
                    if var.variable != None {
                        let name = var.variable.clone().unwrap();
                        if !variables.contains_key(&name) {
                            variables.insert(name, var.is_negated);
                        } else if variables[&name] != var.is_negated {
                            if !removable.contains(&i) {
                                removable.push(i);
                            }
                        }
                    }
                }
            }
        }

        // sort the indices we remove from greatest to least so that we don't go past the length of the vec
        removable.sort_by(|a, b| {
            if a.cmp(b) == std::cmp::Ordering::Less {
                return std::cmp::Ordering::Greater;
            } else {
                return std::cmp::Ordering::Less;
            }
        });

        // remove unecessary clauses
        for i in removable.iter() {
            check = true;
            self.variables.remove(*i);
        }

        // clean the rest of the clauses
        for var in self.variables.iter_mut() {
            check |= var.clean_cnf();
        }

        return check;
    }

    // convert the object model to cnf
    pub fn to_cnf(&mut self) {
        // set the initial check to true
        let mut check = true;

        // while we have made changes, keep looping
        while check {
            check = false;
            check |= self.demorgans();
            check |= self.distributive();
            check |= self.clean_cnf();
        }

        // loop through all variables and get rid of duplicated variables in clauses
        for variable in self.variables.iter_mut() {
            // create a vec to store which indices to be removed
            let mut removable = Vec::new();

            // loop through the clause
            for (i, var) in variable.variables.iter().enumerate() {
                // if the variable is a named variable
                if var.variable != None {
                    // loop througb the rest of the clause and check for duplicates
                    for j in (i + 1)..variable.variables.len() {
                        if variable.variables[j].variable != None {
                            if var
                                .variable
                                .clone()
                                .unwrap()
                                .eq(&variable.variables[j].variable.clone().unwrap())
                                && var.is_negated == variable.variables[j].is_negated
                            {
                                removable.push(j);
                                break;
                            }
                        }
                    }
                }
            }

            // sort the indices we remove from greatest to least so that we don't go past the length of the vec
            removable.sort_by(|a, b| {
                if a.cmp(b) == std::cmp::Ordering::Less {
                    return std::cmp::Ordering::Greater;
                } else {
                    return std::cmp::Ordering::Less;
                }
            });

            // remove all variables that were duplicated
            for i in removable.iter() {
                variable.variables.remove(*i);
            }
        }
    }
}

// Custom fmt for debug prints
impl fmt::Display for PropositionalConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_neg() {
        let con = PropositionalConnection::new(Operator::NONE, true, Some("a".to_string()));

        assert_eq!(con.print_string(), "¬a");
        assert_eq!(con.operator, Operator::NONE);
        assert_eq!(con.variable.unwrap(), "a");
        assert_eq!(con.variables.len(), 0);
    }

    #[test]
    fn test_and_group() {
        let mut con = PropositionalConnection::new(Operator::AND, false, None);
        let a = PropositionalConnection::new(Operator::NONE, true, Some("a".to_string()));
        let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
        let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));

        con.variables.push(a);
        con.variables.push(b);
        con.variables.push(c);

        assert_eq!(con.print_string(), "(¬a ∧ b ∧ c)");
        assert_eq!(con.operator, Operator::AND);
        assert_eq!(con.variable, None);
        assert_eq!(con.variables.len(), 3);
    }

    #[test]
    fn test_nested_groups() {
        let mut con = PropositionalConnection::new(Operator::AND, false, None);
        let mut con2 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con3 = PropositionalConnection::new(Operator::OR, true, None);
        let a = PropositionalConnection::new(Operator::NONE, false, Some("a".to_string()));
        let not_a = PropositionalConnection::new(Operator::NONE, true, Some("a".to_string()));
        let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
        let not_b = PropositionalConnection::new(Operator::NONE, true, Some("b".to_string()));
        let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));

        con3.variables.push(a);
        con3.variables.push(not_b);

        con2.variables.push(not_a);
        con2.variables.push(b);
        con2.variables.push(c);

        con.variables.push(con2);
        con.variables.push(con3);

        assert_eq!(con.print_string(), "((¬a ∧ b ∧ c) ∧ ¬(a ∨ ¬b))");
    }

    #[test]
    fn test_demorgans_law() {
        let mut con = PropositionalConnection::new(Operator::AND, true, None);
        let a = PropositionalConnection::new(Operator::NONE, true, Some("a".to_string()));
        let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
        let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));

        con.variables.push(a);
        con.variables.push(b);
        con.variables.push(c);

        assert_eq!(con.print_string(), "¬(¬a ∧ b ∧ c)");
        con.demorgans();
        assert_eq!(con.print_string(), "(a ∨ ¬b ∨ ¬c)");
    }

    #[test]
    fn test_distributive_law() {
        let mut con = PropositionalConnection::new(Operator::OR, false, None);
        let mut con2 = PropositionalConnection::new(Operator::AND, false, None);
        let a = PropositionalConnection::new(Operator::NONE, false, Some("a".to_string()));
        let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
        let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));

        con2.variables.push(b);
        con2.variables.push(c);

        con.variables.push(a);
        con.variables.push(con2);

        assert_eq!(con.print_string(), "(a ∨ (b ∧ c))");
        con.distributive();
        assert_eq!(con.print_string(), "((a ∨ b) ∧ (a ∨ c))");
    }

    #[test]
    fn test_distributive_law_separated() {
        let mut con = PropositionalConnection::new(Operator::OR, false, None);
        let mut con2 = PropositionalConnection::new(Operator::AND, false, None);
        let a = PropositionalConnection::new(Operator::NONE, false, Some("a".to_string()));
        let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
        let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));
        let d = PropositionalConnection::new(Operator::NONE, false, Some("d".to_string()));

        con2.variables.push(c);
        con2.variables.push(d);

        con.variables.push(a);
        con.variables.push(b);
        con.variables.push(con2);

        assert_eq!(con.print_string(), "(a ∨ b ∨ (c ∧ d))");
        con.distributive();
        assert_eq!(con.print_string(), "((a ∨ b ∨ c) ∧ (a ∨ b ∨ d))");
    }

    #[test]
    fn test_cnf_simple() {
        let mut con = PropositionalConnection::new(Operator::AND, false, None);
        let mut con1 = PropositionalConnection::new(Operator::OR, false, None);
        let mut con11 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con12 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con2 = PropositionalConnection::new(Operator::OR, false, None);
        let mut con21 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con22 = PropositionalConnection::new(Operator::AND, false, None);
        let a = PropositionalConnection::new(Operator::NONE, false, Some("a".to_string()));
        let not_a = PropositionalConnection::new(Operator::NONE, true, Some("a".to_string()));
        let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
        let not_b = PropositionalConnection::new(Operator::NONE, true, Some("b".to_string()));
        let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));
        let not_c = PropositionalConnection::new(Operator::NONE, true, Some("c".to_string()));
        let d = PropositionalConnection::new(Operator::NONE, false, Some("d".to_string()));
        let not_d = PropositionalConnection::new(Operator::NONE, true, Some("d".to_string()));

        con11.variables.push(a);
        con11.variables.push(not_d);

        con12.variables.push(not_a);
        con12.variables.push(d);

        con1.variables.push(con11);
        con1.variables.push(con12);

        con21.variables.push(b);
        con21.variables.push(c);

        con22.variables.push(not_b);
        con22.variables.push(not_c);

        con2.variables.push(con21);
        con2.variables.push(con22);

        con.variables.push(con1);
        con.variables.push(con2);

        assert_eq!(
            con.print_string(),
            "(((a ∧ ¬d) ∨ (¬a ∧ d)) ∧ ((b ∧ c) ∨ (¬b ∧ ¬c)))"
        );
        con.to_cnf();
        assert_eq!(
            con.print_string(),
            "((¬a ∨ ¬d) ∧ (d ∨ a) ∧ (¬b ∨ c) ∧ (¬c ∨ b))"
        );
    }

    #[test]
    fn test_cnf_simple_part_2() {
        let mut con = PropositionalConnection::new(Operator::AND, false, None);
        let mut con3 = PropositionalConnection::new(Operator::OR, false, None);
        let mut con31 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con32 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con33 = PropositionalConnection::new(Operator::AND, false, None);
        let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
        let not_b = PropositionalConnection::new(Operator::NONE, true, Some("b".to_string()));
        let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));
        let not_c = PropositionalConnection::new(Operator::NONE, true, Some("c".to_string()));
        let e = PropositionalConnection::new(Operator::NONE, false, Some("e".to_string()));
        let not_e = PropositionalConnection::new(Operator::NONE, true, Some("e".to_string()));
        let g = PropositionalConnection::new(Operator::NONE, false, Some("g".to_string()));
        let not_g = PropositionalConnection::new(Operator::NONE, true, Some("g".to_string()));

        con31.variables.push(c.clone());
        con31.variables.push(not_g);
        con31.variables.push(not_b.clone());
        con31.variables.push(not_e.clone());

        con32.variables.push(not_c);
        con32.variables.push(g.clone());
        con32.variables.push(not_b);
        con32.variables.push(not_e);

        con33.variables.push(c);
        con33.variables.push(g);
        con33.variables.push(b);
        con33.variables.push(e);

        con3.variables.push(con31);
        con3.variables.push(con32);
        con3.variables.push(con33);

        con.variables.push(con3);

        assert_eq!(
            con.print_string(),
            "((c ∧ ¬g ∧ ¬b ∧ ¬e) ∨ (¬c ∧ g ∧ ¬b ∧ ¬e) ∨ (c ∧ g ∧ b ∧ e))"
        );
        con.to_cnf();
        assert_eq!(
            con.print_string(),
            "((c ∨ g) ∧ (c ∨ g ∨ ¬b) ∧ (c ∨ g ∨ ¬e) ∧ (c ∨ ¬b) ∧ (c ∨ ¬b ∨ ¬g) ∧ (c ∨ ¬b) ∧ (c ∨ ¬b ∨ ¬e) ∧ (c ∨ ¬e) ∧ (c ∨ ¬e ∨ ¬g) ∧ (c ∨ ¬e ∨ ¬b) ∧ (c ∨ ¬e) ∧ (g ∨ ¬c ∨ ¬b) ∧ (g ∨ ¬c ∨ ¬e) ∧ (g ∨ c) ∧ (g ∨ ¬b) ∧ (g ∨ ¬e) ∧ (g ∨ ¬b ∨ c) ∧ (g ∨ ¬b) ∧ (g ∨ ¬b ∨ ¬e) ∧ (g ∨ ¬e ∨ c) ∧ (g ∨ ¬e ∨ ¬b) ∧ (g ∨ ¬e) ∧ (b ∨ ¬c ∨ ¬g) ∧ (b ∨ ¬c ∨ ¬e) ∧ (b ∨ g ∨ c) ∧ (b ∨ g ∨ ¬e) ∧ (b ∨ ¬e ∨ c) ∧ (b ∨ ¬e ∨ ¬g) ∧ (b ∨ ¬e) ∧ (e ∨ ¬c ∨ ¬g) ∧ (e ∨ ¬c ∨ ¬b) ∧ (e ∨ g ∨ c) ∧ (e ∨ g ∨ ¬b) ∧ (e ∨ ¬b ∨ c) ∧ (e ∨ ¬b ∨ ¬g) ∧ (e ∨ ¬b))"
        );
    }

    #[test]
    fn test_cnf_complete() {
        let mut con = PropositionalConnection::new(Operator::AND, false, None);
        let mut con1 = PropositionalConnection::new(Operator::OR, false, None);
        let mut con11 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con12 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con2 = PropositionalConnection::new(Operator::OR, false, None);
        let mut con21 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con22 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con3 = PropositionalConnection::new(Operator::OR, false, None);
        let mut con31 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con32 = PropositionalConnection::new(Operator::AND, false, None);
        let mut con33 = PropositionalConnection::new(Operator::AND, false, None);
        let a = PropositionalConnection::new(Operator::NONE, false, Some("a".to_string()));
        let not_a = PropositionalConnection::new(Operator::NONE, true, Some("a".to_string()));
        let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
        let not_b = PropositionalConnection::new(Operator::NONE, true, Some("b".to_string()));
        let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));
        let not_c = PropositionalConnection::new(Operator::NONE, true, Some("c".to_string()));
        let d = PropositionalConnection::new(Operator::NONE, false, Some("d".to_string()));
        let not_d = PropositionalConnection::new(Operator::NONE, true, Some("d".to_string()));
        let e = PropositionalConnection::new(Operator::NONE, false, Some("e".to_string()));
        let not_e = PropositionalConnection::new(Operator::NONE, true, Some("e".to_string()));
        let g = PropositionalConnection::new(Operator::NONE, false, Some("g".to_string()));
        let not_g = PropositionalConnection::new(Operator::NONE, true, Some("g".to_string()));

        con11.variables.push(a);
        con11.variables.push(not_d);

        con12.variables.push(not_a);
        con12.variables.push(d);

        con1.variables.push(con11);
        con1.variables.push(con12);

        con21.variables.push(b.clone());
        con21.variables.push(c.clone());

        con22.variables.push(not_b.clone());
        con22.variables.push(not_c.clone());

        con2.variables.push(con21);
        con2.variables.push(con22);

        con31.variables.push(c.clone());
        con31.variables.push(not_g);
        con31.variables.push(not_b.clone());
        con31.variables.push(not_e.clone());

        con32.variables.push(not_c);
        con32.variables.push(g.clone());
        con32.variables.push(not_b);
        con32.variables.push(not_e);

        con33.variables.push(c);
        con33.variables.push(g);
        con33.variables.push(b);
        con33.variables.push(e);

        con3.variables.push(con31);
        con3.variables.push(con32);
        con3.variables.push(con33);

        con.variables.push(con1);
        con.variables.push(con2);
        con.variables.push(con3);

        // println!("Test Print: {}", con);
        con.to_cnf();
        // println!("Test Print: {}", con);
    }
}
