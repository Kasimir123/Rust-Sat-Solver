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
    pub var_sets: [[VarUnsatGroup; 325]; 75],
}

impl VarUnsatGroups {
    pub fn new(variable_connections: &Vec<Vec<usize>>) -> Self {
        let mut var_lists: Vec<LinkedList<usize>> = Vec::new();
        let mut var_sets = [[VarUnsatGroup::new(); 325]; 75];
        for i in 0..variable_connections.len() {
            let mut var_list: LinkedList<usize> = LinkedList::<usize>::with_capacity(325);
            for j in variable_connections[i].iter() {
                let node: Node<usize> = var_list.push_head(*j);
                var_sets[i][*j].node = Some(node);
            }
            var_lists.push(var_list);
        }
        VarUnsatGroups {
            var_lists,
            var_sets
        }
    }
}

