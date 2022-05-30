use crate::connections::Connection;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LearnedClause {
    pub lbd: usize,
    pub lits: Vec<Connection>,
    pub index: usize,
}

impl LearnedClause {
    pub fn new(lbd: usize, mut lits: Vec<Connection>, index: usize) -> Self {
        lits.sort();
        LearnedClause {
            lbd,
            lits,
            index
        }
    }
}
