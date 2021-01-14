use super::*;
use rand::random;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct Node {
    pub board: Board,
    pub color: Cell,
    pub children: Vec<Rc<RefCell<Node>>>,
    pub parent: Option<Weak<RefCell<Node>>>,
    pub player_move: Option<PlayerMove>,
    pub leaf: bool,

    pub nwins: u64,
    pub nvisits: u64,
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
        let rc = Rc::new(RefCell::new(node));
        rc
    }

    //     pub fn selection(noderef: NodeRef, my_color: Cell) {
    //         let mut current = noderef;
    //         loop {
    //             let rc = current.clone();
    //             let node = rc.borrow();

    //             if node.children.is_empty() || node.leaf {
    //                 // ...
    //                 return;
    //             }

    //             if node.children.is_empty() {
    //                 Node::expansion(noderef, node.color);
    //                 return;
    //             } else {
    //                 // not expandable
    //                 let mut chosen_child = node.children.first().unwrap().clone();
    //                 if node.color == my_color {
    //                     let mut max_ucb = chosen_child.borrow().ucb;
    //                     for ch in node.children.iter() {
    //                         let ucb = ch.borrow().ucb;
    //                         if ucb > max_ucb {
    //                             chosen_child = ch.clone();
    //                             max_ucb = ucb;
    //                         }
    //                     }
    //                 } else {
    //                     let mut min_lcb = chosen_child.borrow().lcb;
    //                     for ch in node.children.iter() {
    //                         let lcb = ch.borrow().ucb;
    //                         if lcb > min_lcb {
    //                             chosen_child = ch.clone();
    //                             min_lcb = lcb;
    //                         }
    //                     }
    //                 }
    //                 current = chosen_child;
    //             }
    //         }
    //     }

    // pub fn expansion(noderef: NodeRef, color: Cell) {
    //     let node = noderef.borrow();
    //     assert!(!node.children.is_empty());
    //     let allowed_moves = node.board.allowed_moves(color);
    //     for player_move in allowed_moves.iter() {
    //         let child_node = Node {
    //             board: node.board.with_move(player_move, color),
    //             children: vec![],
    //             color: !color,
    //             lcb: 0,
    //             ucb: 0,
    //             leaf: false,
    //             parent: Some(Rc::downgrade(&noderef)),
    //             nvisits: 0,
    //             nwins: 0,
    //             player_move: Some(player_move.clone()),
    //         };
    //         child_node.simulate();
    //         child_node.backpropagate();
    //     }
    // }

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

//             selected = node.children.first().unwrap().clone();
//             let mut best_score = {
//                 let fst = selected.borrow();
//                 let (lcb, ucb) = get_LCB_UCB(
//                     node.nvisits,
//                     fst.nwins,
//                     fst.nvisits,
//                     exploitation_value,
//                 );
//                 if node.color == root_color {
//                     ucb
//                 } else {
//                     lcb
//                 }
//             };

//             for ch in node.children.iter() {
//                 let child = ch.borrow();
//                 let (lcb, ucb) = get_LCB_UCB(
//                     node.nvisits,
//                     child.nwins,
//                     child.nvisits,
//                     exploitation_value,
//                 );
//                 if node.color == root_color && ucb > best_score {
//                     best_score = ucb;
//                     selected = ch.clone();
//                 } else if node.color != root_color && lcb < best_score {
//                     best_score = lcb;
//                     selected = ch.clone();
//                 }
//             }

            let mut max_score = f64::MIN;

            for ch in node.children.iter() {
                let child = ch.borrow();
                let score = uct_score(
                    node.nvisits,
                    child.nwins,
                    child.nvisits,
                    exploitation_value,
                );
                if score > max_score {
                    max_score = score;
                    selected = ch.clone();
                }
            }
        }
        selected
    }

    pub fn expansion(noderef: NodeRef, _is_anti: bool) -> NodeRef {
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

                let rc = Rc::new(RefCell::new(child_node));
                node.children.push(rc);
            }

            let idx = random::<usize>() % node.children.len();
            node.children[idx].clone()
        }
    }

    pub fn simulate(
        noderef: NodeRef,
        is_anti: bool,
        is_depth_even: bool,
        bot_color: Cell,
    ) -> EndState {
        let node = noderef.borrow();
        Board::sim_with_sev(
            node.board,
            node.color,
            is_anti,
            is_depth_even,
            bot_color,
        )
        // Board::simauto(node.board, node.color, is_anti)
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

    // pub fn calc_minimax(noderef: NodeRef, my_color: Cell) {
    //     let mut node = noderef.borrow_mut();
    //     let mut next_minimax_child = node.minimax_child;
    //     if node.children.is_empty() || node.nvisits < node.backup_threshold {
    //         node.minimax_child = Rc::downgrade(&noderef);
    //         return;
    //     }
    //     let mut max_value = f64::MIN;
    //     let mut min_value = f64::MAX;

    //     for ch in node.children.iter() {
    //         let child = ch.borrow();
    //         if node.color == my_color && child.mean > max_value {
    //             if let Some(mm_child) = child.minimax_child.upgrade() {
    //                 let mm_child = mm_child.borrow();
    //                 max_value = mm_child.mean;
    //                 next_minimax_child = mm_child.minimax_child.clone();
    //             }
    //         } else if node.color != my_color && child.mean < min_value {
    //             if let Some(mm_child) = child.minimax_child.upgrade() {
    //                 let mm_child = mm_child.borrow();
    //                 min_value = mm_child.mean;
    //                 next_minimax_child = mm_child.minimax_child.clone();
    //             }
    //         }
    //     }
    //     node.minimax_child = next_minimax_child;
    // }

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
