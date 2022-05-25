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
    pub var_sets: Vec<Vec<VarUnsatGroup>>,
}

impl VarUnsatGroups {
    pub fn new(variable_connections: &Vec<Vec<usize>>) -> Self {
        let mut var_lists: Vec<LinkedList<usize>> = Vec::<LinkedList<usize>>::with_capacity(250);
        let mut var_sets: Vec<Vec<VarUnsatGroup>> = Vec::<Vec<VarUnsatGroup>>::with_capacity(250);
        for i in 0..variable_connections.len() {
            let mut var_list: LinkedList<usize> = LinkedList::<usize>::with_capacity(1065);
            let mut var_set: Vec<VarUnsatGroup> = Vec::<VarUnsatGroup>::with_capacity(1065);
            for _i in 0..1065 {
                var_set.push(VarUnsatGroup::new());
            }
            for group in variable_connections[i].iter() {
                let node: Node<usize> = var_list.push_head(*group);
                let mut var_unsat_group = VarUnsatGroup::new();
                var_unsat_group.node = Some(node);
                var_set[*group] = var_unsat_group;
            }
            var_lists.push(var_list);
            var_sets.push(var_set);
        }
        VarUnsatGroups {
            var_lists,
            var_sets,
        }
    }
}

