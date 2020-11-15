use crate::{board::Board, utils::*};
use crossbeam::channel::{select, unbounded};
use rand::thread_rng;
use rayon::prelude::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    io::Write,
    rc::Rc,
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
    parent: Option<NodeRef>,
    player_move: Option<PlayerMove>,
    leaf: bool,
}

type NodeRef = Rc<RefCell<Node>>;

impl Node {
    fn new(board: Board, color: Cell) -> NodeRef {
        let node = Node {
            board,
            color,
            nwins: 0,
            nvisits: 0,
            children: Vec::new(),
            parent: None,
            player_move: None,
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
                println!("sc:{}", score);
                if score > max_score {
                    max_score = score;
                    noderef = ch.clone();
                    println!("sw");
                }
            }
        }
        noderef
    }

    fn expansion(noderef: NodeRef) -> NodeRef {
        let mut node = noderef.borrow_mut();
        assert_eq!(node.children.len(), 0);
        let allowed = node.board.allowed_moves(node.color);
        if allowed.len() > 0 {
            for player_move in allowed {
                let child_node = Node {
                    color: node.color.opposite(),
                    board: node.board.with_move(&player_move, node.color),
                    parent: Some(noderef.clone()),
                    nwins: 0,
                    nvisits: 0,
                    children: vec![],
                    player_move: Some(player_move),
                    leaf: false,
                };
                node.children.push(Rc::new(RefCell::new(child_node)));
            }
        } else {
            node.leaf = true;
        }
        noderef.clone()
    }

    fn back_propagate(mut noderef: NodeRef, winresult: EndState) {
        loop {
            {
                let mut node = noderef.borrow_mut();
                node.nvisits += 1;
                if winresult.won(node.color) {
                    node.nwins += 1;
                }
            }
            if let Some(parent) = noderef.clone().borrow().parent.clone() {
                noderef = parent;
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
}

impl MCTSBot {
    pub fn new(arg_matches: &clap::ArgMatches) -> Self {
        let black_hole = Chan::read().coord();
        let my_color = Chan::read().color();

        let is_anti = !arg_matches.is_present("no_anti")
            && !std::env::var("NO_ANTI")
                .map(|it| it == "1")
                .unwrap_or(false);

        let board = Board::initial(black_hole);
        let current_color = Cell::Black;
        let allowed_moves = board.allowed_moves(current_color);

        let move_maxtime = arg_matches
            .value_of("time_limit")
            .map(str::to_string)
            .or(std::env::var("MAX_TIME").ok())
            .map(|it| it.parse::<u64>().unwrap())
            .unwrap_or(300);

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

        log!(bot, "black hole: {:?}", black_hole.to_ab());
        log!(bot, "my color: {:?}", my_color);
        log!(bot, "anti reversi mode: {}", is_anti);
        log!(bot, "move timeout: {}\n\nBEGIN:", move_maxtime);

        bot
    }

    fn mcts(&self) -> PlayerMove {
        let tree = Node::new(self.board, self.my_color);
        let timer = Instant::now();

        while timer.elapsed() < self.move_maxtime {
            let n = Node::selection(tree.clone());
            println!("s");
            let expanded = Node::expansion(n);
            let win = expanded.borrow().simulate(self.is_anti);
            Node::back_propagate(expanded, win);
            println!("p");
        }
        let best_node = tree.borrow().best_child();
        best_node.clone().borrow().player_move.clone().unwrap()
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
