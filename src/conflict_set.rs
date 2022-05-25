pub struct ConflictSet {
    pub var_list: [[usize; 250]; 250],
    pub list_len: [usize; 250],
    pub var_set: [[bool; 250]; 250],
}

impl ConflictSet {
    pub fn new() -> Self {
        ConflictSet {
            var_list: [[0; 250]; 250],
            list_len: [0; 250],
            var_set: [[false; 250]; 250],
        }
    }
}
