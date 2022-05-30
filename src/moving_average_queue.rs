pub struct MAQNode {
    pub val: f64,
    pub next: usize,
    pub prev: usize,
}

impl MAQNode {
    pub fn new() -> Self {
        let val: f64 = f64::MAX;
        let next: usize = usize::MAX;
        let prev: usize = usize::MAX;
        MAQNode {
            val,
            next,
            prev,
        }
    }
}

pub struct MovingAverageQueue {
    pub head: usize,
    pub tail: usize,
    pub nodes: Vec<MAQNode>,
    pub switch: bool,
    pub open: usize,
    pub len: usize,
}

impl MovingAverageQueue {
    pub fn new() -> Self {
        let head: usize = usize::MAX;
        let tail: usize = usize::MAX;
        let mut nodes: Vec<MAQNode> = Vec::new();
        for _i in 0..500 {
            nodes.push(MAQNode::new());
        }
        let switch = false;
        let open: usize = 0; // 0 will be the first to be removed
        let len: usize = 0;
        MovingAverageQueue {
            head,
            tail,
            nodes,
            switch,
            open,
            len,
        }
    }
    pub fn e(&mut self, val: f64) {
        if self.len == 0 {
            self.head = 0;
            self.tail = 0;
            self.len = 1;
            self.nodes[0].val = val;
        } else if !self.switch {
            self.nodes[self.len].val = val;
            self.nodes[self.len].prev = self.tail;
            self.nodes[self.tail].next = self.len;
            self.tail = self.len;
            self.len += 1;
            if self.len == 500 {
                self.switch = true;
            }
        } else {
            self.nodes[self.open].val = val;
            self.nodes[self.open].next = usize::MAX;
            self.nodes[self.open].prev = self.tail;
            self.nodes[self.tail].next = self.open;
            self.tail = self.open;
            self.len += 1;
        }
    }
    pub fn d(&mut self) -> f64 {
        let val = self.nodes[self.head].val;
        self.open = self.head;
        // self.nodes[self.nodes[self.head].next].prev = usize::MAX;
        self.head = self.nodes[self.head].next;
        self.len -= 1;
        val
    }
}
