use  deepmesa::lists::LinkedList;
use deepmesa::lists::linkedlist::Node;

// if there's a problem, it's probably this derivation; don't know why this is needed
#[derive(Copy, Clone)]
pub struct VarUnsatGroup {
    pub is_unsat: bool,
    pub node: Option<Node<usize>>,
}

impl VarUnsatGroup {
    pub fn new() -> Self {
        VarUnsatGroup {
            is_unsat: true,
            node: None,
        }
    }
}

pub struct VarUnsatGroups {
    pub var_lists: Vec<LinkedList<usize>>,
    pub var_sets: [[VarUnsatGroup; 218]; 50],
}

impl VarUnsatGroups {
    pub fn new(num_groups: usize) -> Self {
        let mut var_lists: Vec<LinkedList<usize>> = Vec::new();
        let mut var_sets = [[VarUnsatGroup::new(); 218]; 50];
        for i in 0..50 {
            let mut var_list: LinkedList<usize> = LinkedList::<usize>::with_capacity(218);
            for j in 0..num_groups {
                let node: Node<usize> = var_list.push_head(j);
                var_sets[i][j].node = Some(node);
            }
            var_lists.push(var_list);
        }
        VarUnsatGroups {
            var_lists,
            var_sets
        }
    }
}

