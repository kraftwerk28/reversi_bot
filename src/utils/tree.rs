use super::*;
use rand::{thread_rng, Rng};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct Node {
    pub board: Board,
    pub color: Cell,
    pub nwins: u64,
    pub nvisits: u64,
    pub children: Vec<Rc<RefCell<Node>>>,
    pub parent: Option<Weak<RefCell<Node>>>,
    pub player_move: Option<PlayerMove>,
    pub leaf: bool,
}

pub type NodeRef = Rc<RefCell<Node>>;

impl Node {
    pub fn new(
        board: Board,
        color: Cell,
        player_move: Option<PlayerMove>,
    ) -> NodeRef {
        let node = Node {
            board,
            color,
            nwins: 0,
            nvisits: 0,
            children: Vec::new(),
            parent: None,
            player_move,
            leaf: false,
        };
        Rc::new(RefCell::new(node))
    }

    pub fn selection(mut noderef: NodeRef, exploitation_value: f64) -> NodeRef {
        loop {
            let rc = noderef.clone();
            let borrowed = rc.borrow();
            if borrowed.children.is_empty() || borrowed.leaf {
                break;
            }
            let nvisits = borrowed.nvisits;
            let mut max_score = f64::MIN;
            for ch in borrowed.children.iter() {
                let child = ch.borrow();
                let score = uct_score(
                    nvisits,
                    child.nwins,
                    child.nvisits,
                    exploitation_value,
                );
                if score > max_score {
                    max_score = score;
                    noderef = ch.clone();
                }
            }
        }
        noderef
    }

    pub fn expansion(noderef: NodeRef) -> NodeRef {
        let mut rng = thread_rng();

        let mut node = noderef.borrow_mut();
        assert!(node.children.is_empty());
        let allowed = node.board.allowed_moves(node.color);

        if allowed.is_empty() {
            node.leaf = true;
            noderef.clone()
        } else {
            for player_move in allowed.iter() {
                let color = node.color.opposite();
                let board = node.board.with_move(&player_move, node.color);

                let child_node = Node {
                    color,
                    board,
                    nwins: 0,
                    nvisits: 0,
                    parent: Some(Rc::downgrade(&noderef)),
                    children: Vec::new(),
                    player_move: Some(player_move.clone()),
                    leaf: false,
                };
                let noderc = Rc::new(RefCell::new(child_node));
                node.children.push(noderc);
            }
            let ind = rng.gen_range(0, allowed.len());
            node.children[ind].clone()
        }
    }

    pub fn back_propagate(mut noderef: NodeRef, winresult: EndState) {
        loop {
            {
                let mut node = noderef.borrow_mut();
                node.nvisits += 1;
                if winresult.won(node.color.opposite()) {
                    node.nwins += 1;
                }
            }
            let cloned = noderef.clone();
            if let Some(parent) = &cloned.borrow().parent {
                noderef = parent.upgrade().unwrap();
            } else {
                break;
            };
        }
    }

    pub fn simulate(&self, is_anti: bool) -> EndState {
        Board::simauto(self.board, self.color, is_anti)
    }

    #[allow(dead_code)]
    pub fn best_child(&self) -> NodeRef {
        let mut best_node = self.children[0].clone();
        let mut best_score = 0f64;
        for ch in self.children.iter() {
            let child = ch.borrow();
            let score = child.nwins as f64 / child.nvisits as f64;
            if score > best_score {
                best_score = score;
                best_node = ch.clone();
            }
        }
        best_node
    }

    #[allow(dead_code)]
    pub fn repr_node(nr: &NodeRef, indent: usize) -> String {
        let n = nr.borrow();
        let indstr = " ".repeat(indent * 2);
        let nv = n
            .children
            .iter()
            .map(|n| Node::repr_node(n, indent + 1))
            .collect::<Vec<_>>()
            .join(",\n");
        format!(
            "{}Node({}/{}; [\n{}\n{}])",
            indstr, n.nwins, n.nvisits, nv, indstr
        )
    }

    #[allow(dead_code)]
    pub fn score(&self) -> f64 {
        if self.nvisits == 0 {
            f64::MIN
        } else {
            self.nwins as f64 / self.nvisits as f64
        }
    }
}
