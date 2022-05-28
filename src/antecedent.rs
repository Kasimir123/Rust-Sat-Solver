pub struct Antecedent {
    pub is_uc: bool,
    pub antecedent: Option<usize>,
    pub d: usize,
}

impl Antecedent {
    pub fn new() -> Self {
        let is_uc = false;
        let antecedent = None;
        let d = 0;
        Antecedent {
            is_uc,
            antecedent,
            d
        }
    }
}
