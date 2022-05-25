pub struct ConflictSet {
    pub var_list: [[usize; 75]; 75],
    pub list_len: [usize; 75],
    pub var_set: [[bool; 75]; 75],
}

impl ConflictSet {
    pub fn new() -> Self {
        ConflictSet {
            var_list: [[0; 75]; 75],
            list_len: [0; 75],
            var_set: [[false; 75]; 75],
        }
    }
}
