use crate::{board::Board, utils::*};
use crossbeam::channel::unbounded;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use std::{
    cell::RefCell,
    io::Write,
    rc::{Rc, Weak},
    thread,
    time::{Duration, Instant},
};

pub struct MCTSBot {
    board: Board,
    log_file: LogFile,
    move_maxtime: Duration,
    my_color: Cell,
    win_state: EndState,
    current_color: Cell,
    allowed_moves: AllowedMoves,
    is_anti: bool,
}

struct Node {
    board: Board,
    color: Cell,
    nwins: u64,
    nvisits: u64,
    children: Vec<NodeRef>,
    parent: Option<WeakNodeRef>,
    player_move: Option<PlayerMove>,
    leaf: bool,
}

type NodeRef = Rc<RefCell<Node>>;
type WeakNodeRef = Weak<RefCell<Node>>;

impl Node {
    fn new(
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

    fn selection(mut noderef: NodeRef) -> NodeRef {
        loop {
            let node = noderef.clone();
            if node.borrow().children.is_empty() || node.borrow().leaf {
                break;
            }
            let nvisits = node.borrow().nvisits;
            let mut max_score = f64::MIN;
            for ch in node.borrow().children.iter() {
                let child = ch.borrow();
                let score = uct_score(nvisits, child.nwins, child.nvisits);
                if score > max_score {
                    max_score = score;
                    noderef = ch.clone();
                }
            }
        }
        noderef
    }

    fn expansion(mut noderef: NodeRef) -> NodeRef {
        let mut rng = thread_rng();
        let allowed = {
            let node = noderef.borrow();
            assert_eq!(node.children.len(), 0);
            node.board.allowed_moves(node.color)
        };

        if !allowed.is_empty() {
            let child = {
                let mut node = noderef.borrow_mut();
                for player_move in allowed.iter() {
                    let child_node = Node {
                        color: node.color.opposite(),
                        board: node.board.with_move(&player_move, node.color),
                        parent: Some(Rc::downgrade(&noderef)),
                        nwins: 0,
                        nvisits: 0,
                        children: vec![],
                        player_move: Some(player_move.clone()),
                        leaf: false,
                    };
                    node.children.push(Rc::new(RefCell::new(child_node)));
                }
                node.children[rng.gen_range(0, allowed.len())].clone()
            };
            noderef = child;
        } else {
            let mut node = noderef.borrow_mut();
            node.leaf = true;
        }
        noderef.clone()
    }

    fn back_propagate(mut noderef: NodeRef, winresult: EndState) {
        loop {
            {
                let mut node = noderef.borrow_mut();
                node.nvisits += 1;
                if winresult.won(node.color.opposite()) {
                    node.nwins += 1;
                }
            }
            let node = noderef.clone();
            if let Some(parent) = &node.borrow().parent {
                noderef = parent.upgrade().unwrap();
            } else {
                break;
            };
        }
    }

    fn simulate(&self, is_anti: bool) -> EndState {
        Board::simauto(self.board, self.color, is_anti)
    }

    fn best_child(&self) -> NodeRef {
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

    fn repr_node(nr: &NodeRef, indent: usize) -> String {
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

    fn score(&self) -> f64 {
        if self.nvisits == 0 {
            f64::MIN
        } else {
            self.nwins as f64 / self.nvisits as f64
        }
    }
}

impl MCTSBot {
    pub fn new(arg_matches: &clap::ArgMatches) -> Self {
        let black_hole = Chan::read().coord();
        let my_color = Chan::read().color();

        let is_anti = !arg_matches.is_present("no_anti");

        let board = Board::initial(black_hole);
        let current_color = Cell::Black;
        let allowed_moves = board.allowed_moves(current_color);

        let move_maxtime = arg_matches
            .value_of("time_limit")
            .map(|it| it.parse::<u64>().unwrap())
            .unwrap();

        let bot = Self {
            board,
            my_color,
            current_color,
            win_state: EndState::Unknown,

            allowed_moves,
            log_file: get_logfile(&arg_matches),
            is_anti,
            move_maxtime: Duration::from_millis(move_maxtime),
        };

        log!(bot, "alg: Advanced MCTS");
        log!(bot, "black hole: {:?}", black_hole.to_ab());
        log!(bot, "my color: {:?}", my_color);
        log!(bot, "anti reversi mode: {}", is_anti);
        log!(bot, "move timeout: {}\n\nBEGIN:", move_maxtime);

        bot
    }

    fn mcts(&self) -> PlayerMove {
        let allowed_moves = self.board.allowed_moves(self.my_color);
        let (stop_tx, stop_rx) = unbounded::<()>();

        let tim_thread = thread::spawn({
            let max_time = self.move_maxtime;
            let stop_signals_count = allowed_moves.len();
            move || {
                let timer = Instant::now();
                while timer.elapsed() < max_time {}
                for _ in 0..stop_signals_count {
                    stop_tx.send(()).unwrap();
                }
            }
        });

        let scores = allowed_moves
            .par_iter()
            .map(|pl_move| {
                let new_board = self.board.with_move(pl_move, self.my_color);
                let tree = Node::new(
                    new_board,
                    self.my_color.opposite(),
                    Some(pl_move.clone()),
                );
                loop {
                    let selected = Node::selection(tree.clone());
                    let expanded = Node::expansion(selected);
                    let rollout_result =
                        expanded.borrow().simulate(self.is_anti);
                    Node::back_propagate(expanded, rollout_result);
                    if let Ok(_) = stop_rx.try_recv() {
                        break;
                    }
                }
                let node = tree.borrow();
                (node.score(), node.player_move.clone().unwrap())
            })
            .collect::<Vec<_>>();

        tim_thread.join().unwrap();

        // let tree = Node::new(self.board, self.my_color);
        // let timer = Instant::now();
        // let mut nhits = 0;

        // while timer.elapsed() < self.move_maxtime {
        //     nhits += 1;
        //     let n = Node::selection(tree.clone());
        //     let expanded = Node::expansion(n);
        //     let win = expanded.borrow().simulate(self.is_anti);
        //     Node::back_propagate(expanded, win);
        // }

        // let tree = tree.borrow();

        // log!(self, "total hits: {}", nhits);
        // log!(
        //     self,
        //     "final scores: [{}]",
        //     tree.children
        //         .iter()
        //         .map(|n| {
        //             let n = n.borrow();
        //             format!("{}/{}", n.nwins, n.nvisits)
        //         })
        //         .collect::<Vec<_>>()
        //         .join(", ")
        // );

        // let best_node = tree.best_child().clone();
        // let pl_move = best_node.borrow().player_move.clone().unwrap();
        // pl_move
        let mut max_score = f64::MIN;
        let mut best_move = &scores[0].1;
        for (score, player_move) in scores.iter() {
            if *score > max_score {
                max_score = *score;
                best_move = &player_move
            }
        }
        best_move.clone()
    }
}

impl Bot for MCTSBot {
    fn run(&mut self) {
        loop {
            self.allowed_moves = self.board.allowed_moves(self.current_color);
            self.win_state = wincheck(
                &self.board,
                &self.allowed_moves,
                self.is_anti,
                self.current_color,
            );

            if self.win_state.is_over() {
                break;
            }

            if self.allowed_moves.len() > 0 {
                if self.current_color == self.my_color {
                    let pl_move = self.mcts();
                    log!(self, "my move: {}", pl_move.0.to_ab());
                    self.board.apply_move(&pl_move, self.current_color);
                    Chan::send(CLIMove::Coord(pl_move.0));
                } else {
                    let coord = Chan::read().coord();
                    log!(self, "their move: {}", coord.to_ab());
                    let pl_move = self
                        .allowed_moves
                        .iter()
                        .find(|(ti, _)| *ti == coord)
                        .expect("Not a possible move from opponent")
                        .clone();
                    self.board.apply_move(&pl_move, self.current_color);
                }
            } else {
                if self.current_color == self.my_color {
                    Chan::send(CLIMove::Pass);
                } else {
                    Chan::read();
                }
            }
            self.current_color = self.current_color.opposite();
            log!(self, "{:?}", self.board);
        }
    }

    fn report(&self) {
        log!(
            self,
            "{}",
            match self.win_state {
                EndState::Tie => "Tie!",
                EndState::BlackWon => "Black won!",
                EndState::WhiteWon => "White won!",
                _ => "Game hadn't been completed.",
            }
        );
        if let Some(logfile) = &self.log_file {
            let lck = logfile.lock().unwrap();
            lck.borrow_mut().flush().unwrap();
        }
    }
}
