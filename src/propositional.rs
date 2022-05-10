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

    pub fn demorgans(&mut self) {
        let apply_de_morgan = self.variables.len() > 1 && self.is_negated;

        if apply_de_morgan {
            self.is_negated = false;
            self.operator = if self.operator == Operator::AND {
                Operator::OR
            } else {
                Operator::AND
            };
        }

        for var in self.variables.iter_mut() {
            if apply_de_morgan {
                var.is_negated = !var.is_negated;
            }
            var.demorgans();
        }
    }

    pub fn distributive(&mut self) {
        if self.variable == None {
            if self.operator == Operator::OR && self.variables.len() > 0 {
                let mut left_hand_group = PropositionalConnection::new(Operator::NONE, false, None);

                left_hand_group.variables.push(self.variables[0].clone());

                for i in 1..self.variables.len() {
                    let cur_var = &self.variables[i];
                    if cur_var.operator == Operator::OR || cur_var.operator == Operator::NONE {
                        left_hand_group.variables.push(cur_var.clone());
                    } else {
                        let mut new_and = PropositionalConnection::new(Operator::AND, false, None);
                        for var in self.variables[i].variables.iter() {
                            let mut new_or =
                                PropositionalConnection::new(Operator::OR, false, None);
                            for left_var in left_hand_group.variables.iter() {
                                new_or.variables.push(left_var.clone());
                            }
                            new_or.variables.push(var.clone());

                            new_or.variables.sort_by(|a, b| {
                                if a.operator == b.operator {
                                    return std::cmp::Ordering::Equal;
                                } else if a.operator == Operator::AND {
                                    return std::cmp::Ordering::Greater;
                                } else {
                                    return std::cmp::Ordering::Less;
                                }
                            });

                            new_and.variables.push(new_or);
                        }

                        left_hand_group.variables.clear();

                        new_and.variables.sort_by(|a, b| {
                            if a.operator == b.operator {
                                return std::cmp::Ordering::Equal;
                            } else if a.operator == Operator::AND {
                                return std::cmp::Ordering::Greater;
                            } else {
                                return std::cmp::Ordering::Less;
                            }
                        });

                        left_hand_group.variables.push(new_and);
                    }
                }

                self.variables.clear();

                left_hand_group.variables.sort_by(|a, b| {
                    if a.operator == b.operator {
                        return std::cmp::Ordering::Equal;
                    } else if a.operator == Operator::AND {
                        return std::cmp::Ordering::Greater;
                    } else {
                        return std::cmp::Ordering::Less;
                    }
                });

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

            for var in self.variables.iter_mut() {
                var.distributive();
            }
        }
    }

    pub fn clean_cnf(&mut self) {

        
        let variables = self.variables.clone();
        self.variables.clear();

        if self.operator == Operator::AND {
            for var in variables.iter() {
                if var.operator == Operator::AND {
                    for new_var in var.variables.iter() {
                        self.variables.push(new_var.clone());
                    }
                } else {
                    self.variables.push(var.clone());
                }
            }
        }
        else if self.operator == Operator::OR {
            for var in variables.iter() {
                if var.operator == Operator::OR {
                    for new_var in var.variables.iter() {
                        self.variables.push(new_var.clone());
                    }
                } else {
                    self.variables.push(var.clone());
                }
            }
        }
        else {
            for var in variables.iter() {
                    self.variables.push(var.clone());
            }
        }

        let mut removable = Vec::new();
        for (i, variable) in self.variables.iter_mut().enumerate() {
            let mut variables: HashMap<String, bool> = HashMap::new();
            for var in variable.variables.iter() {
                if var.variable != None {
                    let name = var.variable.clone().unwrap();
                    if !variables.contains_key(&name) {
                        variables.insert(name, var.is_negated);
                    } else if variables[&name] != var.is_negated {
                        removable.push(i);
                        break;
                    }
                }
            }
        }

        for i in removable.iter() {
            if *i < self.variables.len() {
                self.variables.remove(*i);
            }
        }

        for var in self.variables.iter_mut() {
            var.clean_cnf();
        }
    }

    pub fn debug_print(&self) {
        println!("Big guy: {:?} {}", self.operator, self);

        for var in self.variables.iter() {
            println!("Small guy: {:?} {}", var.operator, var);
        }

        for var in self.variables.iter() {
            var.debug_print();
        }
    }

    pub fn to_cnf(&mut self) {
        self.demorgans();
        self.distributive();
        self.clean_cnf();
    }
}

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

        println!("Test Print: {}", con);
        con.to_cnf();
        con.to_cnf();
        con.to_cnf();
        con.to_cnf();
        con.to_cnf();
        con.to_cnf();
        con.to_cnf();
        con.to_cnf();
        con.to_cnf();
        con.to_cnf();
        con.to_cnf();
        println!("Test Print: {}", con);
    }

    // #[test]
    // fn test_cnf_complete() {
    //     let mut con = PropositionalConnection::new(Operator::AND, false, None);
    //     let mut con1 = PropositionalConnection::new(Operator::OR, false, None);
    //     let mut con11 = PropositionalConnection::new(Operator::AND, false, None);
    //     let mut con12 = PropositionalConnection::new(Operator::AND, false, None);
    //     let mut con2 = PropositionalConnection::new(Operator::OR, false, None);
    //     let mut con21 = PropositionalConnection::new(Operator::AND, false, None);
    //     let mut con22 = PropositionalConnection::new(Operator::AND, false, None);
    //     let mut con3 = PropositionalConnection::new(Operator::OR, false, None);
    //     let mut con31 = PropositionalConnection::new(Operator::AND, false, None);
    //     let mut con32 = PropositionalConnection::new(Operator::AND, false, None);
    //     let mut con33 = PropositionalConnection::new(Operator::AND, false, None);
    //     let a = PropositionalConnection::new(Operator::NONE, false, Some("a".to_string()));
    //     let not_a = PropositionalConnection::new(Operator::NONE, true, Some("a".to_string()));
    //     let b = PropositionalConnection::new(Operator::NONE, false, Some("b".to_string()));
    //     let not_b = PropositionalConnection::new(Operator::NONE, true, Some("b".to_string()));
    //     let c = PropositionalConnection::new(Operator::NONE, false, Some("c".to_string()));
    //     let not_c = PropositionalConnection::new(Operator::NONE, true, Some("c".to_string()));
    //     let d = PropositionalConnection::new(Operator::NONE, false, Some("d".to_string()));
    //     let not_d = PropositionalConnection::new(Operator::NONE, true, Some("d".to_string()));
    //     let e = PropositionalConnection::new(Operator::NONE, false, Some("e".to_string()));
    //     let not_e = PropositionalConnection::new(Operator::NONE, true, Some("e".to_string()));
    //     let g = PropositionalConnection::new(Operator::NONE, false, Some("g".to_string()));
    //     let not_g = PropositionalConnection::new(Operator::NONE, true, Some("g".to_string()));

    //     con11.variables.push(a);
    //     con11.variables.push(not_d);

    //     con12.variables.push(not_a);
    //     con12.variables.push(d);

    //     con1.variables.push(con11);
    //     con1.variables.push(con12);

    //     con21.variables.push(b.clone());
    //     con21.variables.push(c.clone());

    //     con22.variables.push(not_b.clone());
    //     con22.variables.push(not_c.clone());

    //     con2.variables.push(con21);
    //     con2.variables.push(con22);

    //     // con31.variables.push(c.clone());
    //     // con31.variables.push(not_g);
    //     // con31.variables.push(not_b.clone());
    //     // con31.variables.push(not_e.clone());

    //     // con32.variables.push(not_c);
    //     // con32.variables.push(g.clone());
    //     // con32.variables.push(not_b);
    //     // con32.variables.push(not_e);

    //     // con33.variables.push(c);
    //     // con33.variables.push(g);
    //     // con33.variables.push(b);
    //     // con33.variables.push(e);

    //     // con3.variables.push(con31);
    //     // con3.variables.push(con32);
    //     // con3.variables.push(con33);

    //     con.variables.push(con1);
    //     con.variables.push(con2);
    //     // con.variables.push(con3);

    //     println!("Test Print: {}", con);
    //     con.to_cnf();
    //     println!("Test Print: {}", con);
    // }
}
