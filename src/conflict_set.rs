pub struct ConflictSet {
    pub var_list: [[usize; 200]; 200],
    pub list_len: [usize; 200],
    pub var_set: [[bool; 200]; 200],
}

impl ConflictSet {
    pub fn new() -> Self {
        ConflictSet{
            var_list: [[0; 200]; 200],
            list_len: [0; 200],
            var_set: [[false; 200]; 200],
        }
    }
}
