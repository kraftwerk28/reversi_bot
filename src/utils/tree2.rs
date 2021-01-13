use super::*;
use rand::random;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct Node {
    pub board: Board,
    pub color: Cell,
    pub nwins: u64,
    pub nvisits: u64,
    pub children: Vec<NodeRef>,
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

    pub fn selection(noderef: NodeRef, exploitation_value: f64) -> NodeRef {
        let root_color = {
            let bor = noderef.borrow();
            bor.color
        };
        let mut selected = noderef;

        loop {
            let rc = selected.clone();
            let node = rc.borrow();

            if node.children.is_empty() || node.leaf {
                break;
            }

            let mut best_score = if node.color == root_color {
                f64::MIN
            } else {
                f64::MAX
            };

            for ch in node.children.iter() {
                let child = ch.borrow();
                let score = uct_score(
                    node.nvisits,
                    child.nwins,
                    child.nvisits,
                    exploitation_value,
                );
                if (node.color == root_color && score > best_score)
                    || (node.color != root_color && score < best_score)
                {
                    best_score = score;
                    selected = ch.clone();
                }
            }
        }
        selected
    }

    pub fn expansion(noderef: NodeRef) -> NodeRef {
        let mut node = noderef.borrow_mut();
        assert!(node.children.is_empty());
        let allowed = node.board.allowed_moves(node.color);

        if allowed.is_empty() {
            node.leaf = true;
            noderef.clone()
        } else {
            let color = !node.color;
            for player_move in allowed.iter() {
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

            let idx = random::<usize>() % node.children.len();
            node.children[idx].clone()
        }
    }

    pub fn simulate(&self, is_anti: bool) -> EndState {
        Board::simauto(self.board, self.color, is_anti)
    }

    pub fn back_propagate(
        noderef: NodeRef,
        winresult: EndState,
        is_anti: bool,
    ) {
        let mut current = noderef;
        loop {
            {
                let mut node = current.borrow_mut();
                node.nvisits += 1;
                let color = if is_anti { !node.color } else { node.color };
                if winresult.won(color) {
                    node.nwins += 1;
                }
            }

            let cloned = current.clone();
            if let Some(parent) = &cloned.borrow().parent {
                current = parent.upgrade().unwrap();
            } else {
                break;
            };
        }
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
