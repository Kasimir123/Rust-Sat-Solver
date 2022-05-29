#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LearnedClause {
    pub lbd: usize,
    pub lits: usize,
    pub index: usize,
}

impl LearnedClause {
    pub fn new(lbd: usize, lits: usize, index: usize) -> Self {
        LearnedClause {
            lbd,
            lits,
            index
        }
    }
}
