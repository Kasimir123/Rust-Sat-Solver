pub struct ConflictSet {
    pub var_list: [[usize; 50]; 50],
    pub list_len: [usize; 50],
    pub var_set: [[bool; 50]; 50],
}

impl ConflictSet {
    pub fn new() -> Self {
        ConflictSet {
            var_list: [[0; 50]; 50],
            list_len: [0; 50],
            var_set: [[false; 50]; 50],
        }
    }
}
