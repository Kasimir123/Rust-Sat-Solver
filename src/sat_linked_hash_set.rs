pub struct SatNode {
    pub val: usize,
    pub next: usize,
    pub prev: usize,
}

impl SatNode {
    pub fn new(val: usize, next: usize, prev: usize) -> Self {
        SatNode { val, next, prev }
    }
}

pub struct SatLinkedHashSet {
    pub heads: Vec<usize>,
    pub var_lists: Vec<Vec<SatNode>>,
    // calling this a set because -1 (or whatever max value is, it's unsigned) could designate !contains
    pub var_sets: Vec<Vec<usize>>,
    pub open_spots: Vec<Vec<usize>>,
    pub open_spots_len: Vec<usize>,
}

impl SatLinkedHashSet {
    pub fn new(variable_connections: &Vec<Vec<usize>>) -> Self {
        let mut var_lists: Vec<Vec<SatNode>> = Vec::<Vec<SatNode>>::with_capacity(250);
        let mut var_sets: Vec<Vec<usize>> = Vec::<Vec<usize>>::with_capacity(250);
        let mut open_spots: Vec<Vec<usize>> = Vec::<Vec<usize>>::with_capacity(250);
        let mut open_spots_len: Vec<usize> = Vec::<usize>::with_capacity(250);
        let mut heads: Vec<usize> = Vec::<usize>::with_capacity(250);
        let num_vars = variable_connections.len();
        for i in 0..num_vars {
            let mut var_list: Vec<SatNode> = Vec::<SatNode>::with_capacity(1065);
            let mut var_set: Vec<usize> = Vec::<usize>::with_capacity(1065);
            let mut var_open_spots: Vec<usize> = Vec::<usize>::with_capacity(1065);
            let mut var_open_spots_len: usize = 0;
            let head: usize = 0;
            for j in 0..1065 {
                var_open_spots.push(1065 - j - 1);
                var_open_spots_len += 1;
                var_list.push(SatNode { val: usize::MAX, next: usize::MAX, prev: usize::MAX });
                var_set.push(usize::MAX);
            }
            let num_cons = variable_connections[i].len();
            for j in 0..num_cons {
                let group = variable_connections[i][j];
                let mut next = j + 1;
                if next == num_cons {
                    next = usize::MAX;
                }
                let node = SatNode::new(group, next, j - 1);
                var_list[j] = node;
                var_set[group] = j;
                var_open_spots_len -= 1;
            }
            var_lists.push(var_list);
            var_sets.push(var_set);
            open_spots.push(var_open_spots);
            open_spots_len.push(var_open_spots_len);
            heads.push(head);
        }
        SatLinkedHashSet {
            heads,
            var_lists,
            var_sets,
            open_spots,
            open_spots_len,
        }
    }
    pub fn update_previous_head_insert(&mut self, var: usize, previous_head: usize, spot_going_in: usize) {
        let mut previous_head_node = &mut self.var_lists[var][previous_head];
        previous_head_node.prev = spot_going_in;
    }
    pub fn update_node_being_replaced(&mut self, var: usize, con: usize, spot_going_in: usize, previous_head: usize) {
        let mut node_being_replaced = &mut self.var_lists[var][spot_going_in];
        node_being_replaced.val = con;
        node_being_replaced.prev = usize::MAX;
        node_being_replaced.next = previous_head;
    }
    pub fn update_head_insert(&mut self, var: usize, spot_going_in: usize) {
        self.heads[var] = spot_going_in;
    }
    pub fn update_var_sets_insert(&mut self, var: usize, con: usize, spot_going_in: usize) {
        self.var_sets[var][con] = spot_going_in;
    }
    pub fn insert(&mut self, var: usize, con: usize) {
        // if var == 5 {
        //     println!("con: {}", con);
        //     let mut node = self.var_lists[var][self.heads[var]].val;
        //     println!("head: {}", node);
        //     let mut next = self.var_lists[var][self.heads[var]].next;
        //     while next != usize::MAX {
        //         node = self.var_lists[var][next].val;
        //         println!("next: {}", node);
        //         if node == 602 {
        //             println!("spot_will_be_open? {}", next);
        //         }
        //         next = self.var_lists[var][next].next;
        //     }
        //     // std::process::exit(1);
        // }
        let previous_head = self.heads[var];
        // if self.open_spots_len[var] == 0 {
        //     println!("{}", var);
        // }
        let spot_going_in = self.open_spots[var][self.open_spots_len[var] - 1];
        self.update_var_sets_insert(var, con, spot_going_in);
        self.open_spots_len[var] -= 1;
        if previous_head != usize::MAX {
            self.update_previous_head_insert(var, previous_head, spot_going_in);
        }
        self.update_node_being_replaced(var, con, spot_going_in, previous_head);
        self.update_head_insert(var, spot_going_in);
    }
    pub fn update_open_spots_remove(&mut self, var: usize, spot_will_be_open: usize) {
        // if self.open_spots_len[var] == 1065 {
        //     println!("{}", var);
        // }
        self.open_spots[var][self.open_spots_len[var]] = spot_will_be_open;
        self.open_spots_len[var] += 1;
    }
    pub fn empty_list(&mut self, var: usize) {
        self.heads[var] = usize::MAX;
    }
    pub fn remove_head(&mut self, var: usize) {
        self.heads[var] = self.var_lists[var][self.heads[var]].next;
        self.var_lists[var][self.heads[var]].prev = usize::MAX;
    }
    pub fn remove_tail(&mut self, var: usize, prior_to_tail: usize) {
        self.var_lists[var][prior_to_tail].next = usize::MAX;
    }
    pub fn connect_neighbors_remove(&mut self, var: usize, prior_node: usize, after_node: usize) {
        self.var_lists[var][prior_node].next = after_node;
        self.var_lists[var][after_node].prev = prior_node;
    }
    pub fn update_neighbors_remove(&mut self, var: usize, spot_will_be_open: usize) {
        let node_being_removed = &self.var_lists[var][spot_will_be_open];
        let is_head = node_being_removed.prev == usize::MAX;
        let is_tail = node_being_removed.next == usize::MAX;
        if is_head && is_tail {
            self.empty_list(var);
        } else if is_head {
            self.remove_head(var);
        } else if is_tail {
            let prior_to_tail = self.var_lists[var][spot_will_be_open].prev;
            self.remove_tail(var, prior_to_tail);
        } else {
            let prior_node = self.var_lists[var][spot_will_be_open].prev;
            let after_node = self.var_lists[var][spot_will_be_open].next;
            self.connect_neighbors_remove(var, prior_node, after_node);
        }
    }
    pub fn reset_node_being_removed(&mut self, var: usize, con: usize) {
        self.var_sets[var][con] = usize::MAX;
    }
    pub fn remove(&mut self, var: usize, con: usize) {
        // if var == 33 {
        //     println!("con: {}", con);
        //     let mut node = self.var_lists[var][self.heads[var]].val;
        //     println!("head: {}", node);
        //     let mut next = self.var_lists[var][self.heads[var]].next;
        //     while next != usize::MAX {
        //         node = self.var_lists[var][next].val;
        //         println!("next: {}", node);
        //         if node == 602 {
        //             println!("spot_will_be_open? {}", next);
        //         }
        //         next = self.var_lists[var][next].next;
        //     }
        //     // std::process::exit(1);
        // }
        let spot_will_be_open = self.var_sets[var][con];
        // if var == 33 {
        //     println!("spot_will_be_open {}", spot_will_be_open);
        //     std::process::exit(1);
        // }
        self.reset_node_being_removed(var, con);
        self.update_open_spots_remove(var, spot_will_be_open);
        self.update_neighbors_remove(var, spot_will_be_open);
    }
    pub fn contains(&mut self, var: usize, con: usize) -> bool {
        self.var_sets[var][con] != usize::MAX
    }
}

