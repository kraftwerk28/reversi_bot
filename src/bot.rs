use crate::{board::Board, sev::*, utils::*};
use clap::ArgMatches;
use rayon::prelude::*;
use std::collections::HashMap;
use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::Write,
    sync::Mutex,
    time::Instant,
};

// A small logging util
macro_rules! log {
    ($self:ident, $($fmtargs:expr),+) => {
        if let Some(log_file) = &$self.log_file {
            let lck = log_file.lock().unwrap();
            let mut writable = lck.borrow_mut();
            writeln!(writable, $($fmtargs),+).unwrap();
        }
    }
}

pub struct Bot {
    board: Board,
    my_color: Cell,
    current_color: Cell,
    win_state: EndState,
    allowed_moves: AllowedMoves,
    max_tree_depth: usize,
    log_file: Option<Mutex<RefCell<File>>>,
    is_anti: bool,
}

struct ScoreTree(Mutex<RefCell<HashMap<usize, (Score, Vec<Score>)>>>);
#[allow(dead_code)]
impl ScoreTree {
    fn new() -> Self {
        Self(Mutex::new(RefCell::new(HashMap::new())))
    }

    fn add_score(&self, depth: usize, score: Score) {
        let lck = self.0.lock().unwrap();
        let mut lck = lck.borrow_mut();
        let (prev_sc, mut prev_scs) =
            lck.get(&depth).unwrap_or(&(0, Vec::new())).clone();
        prev_scs.push(score);
        lck.insert(depth, (prev_sc, prev_scs));
    }

    fn set_best(&self, depth: usize, score: Score) {
        let lck = self.0.lock().unwrap();
        let mut lck = lck.borrow_mut();
        let (_, prev_scs) = lck.get(&depth).unwrap_or(&(0, Vec::new())).clone();
        lck.insert(depth, (score, prev_scs));
    }

    fn retrieve(&self) -> HashMap<usize, (Score, Vec<Score>)> {
        let lck = self.0.lock().unwrap();
        let hm = lck.borrow().clone();
        hm
    }
}

impl Bot {
    pub fn new(arg_matches: &ArgMatches) -> Self {
        let black_hole = Chan::read().coord();
        let my_color = Chan::read().color();

        let log_file = arg_matches
            .value_of("log_file")
            .map(|s| s.to_string())
            .or(std::env::var("LOG").ok())
            .map(|name| {
                OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(name)
                    .map(|f| Mutex::new(RefCell::new(f)))
                    .unwrap()
            });

        let max_tree_depth = arg_matches
            .value_of("max_depth")
            .map(|s| s.to_string())
            .or(std::env::var("TREE").ok())
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(7);

        let is_anti = !arg_matches.is_present("no-anti")
            && !std::env::var("NO_ANTI")
                .map(|it| it == "1")
                .unwrap_or(false);

        let board = Board::initial(black_hole);
        let current_color = Cell::Black;
        let allowed_moves = board.allowed_moves(current_color);

        let bot = Self {
            board,
            my_color,
            current_color,
            win_state: EndState::Unknown,
            allowed_moves,
            max_tree_depth,
            log_file,
            is_anti,
        };

        log!(bot, "black hole: {:?}", black_hole.to_ab());
        log!(bot, "my color: {:?}", my_color);
        log!(bot, "anti reversi mode: {}", is_anti);
        log!(bot, "tree depth: {}\n\nBEGIN:", max_tree_depth);

        bot
    }

    pub fn run(&mut self) {
        loop {
            self.update_allowed();
            self.wincheck();

            match self.win_state {
                EndState::Unknown => {
                    if self.current_color == self.my_color {
                        self.my_move();
                    } else {
                        self.their_move();
                    }
                    self.switch_player();
                }
                EndState::OnePass => {
                    self.pass_move();
                    self.switch_player();
                }
                _ => {
                    break;
                }
            };
            log!(self, "{:?}", self.board);
        }
    }

    fn my_move(&mut self) {
        let time = Instant::now();

        let pl_move = self.run_minimax();
        log!(
            self,
            "my move: {}; {}ms passed",
            pl_move.0.to_ab(),
            time.elapsed().as_millis()
        );
        self.board.apply_move(&pl_move, self.current_color);
        Chan::send(CLIMove::Coord(pl_move.0));
    }

    fn run_minimax(&self) -> PlayerMove {
        let best_eval = {
            let score = if self.is_anti { Score::MAX } else { Score::MIN };
            let tup = (score, self.allowed_moves.first().unwrap());
            Mutex::new(tup)
        };
        let ab = (Score::MIN, Score::MAX);

        self.allowed_moves.par_iter().for_each(|pl_move| {
            let score = self.minimax(
                self.board.with_move(pl_move, self.my_color),
                self.max_tree_depth,
                ab,
                self.my_color.opposite(),
            );
            let mut lck = best_eval.lock().unwrap();
            if (self.is_anti && score < lck.0)
                || (!self.is_anti && score > lck.0)
            {
                *lck = (score, pl_move);
            }
        });
        let best_eval = best_eval.lock().unwrap();
        best_eval.1.clone()
    }

    fn minimax(
        &self,
        board: Board,
        depth: usize,
        ab: AlphaBeta,
        color: Cell,
    ) -> Score {
        let allowed_moves = board.allowed_moves(color);

        if depth <= 0 || allowed_moves.is_empty() {
            return static_eval_with_weights_2(&board, self.my_color);
        }

        let is_maxing = if self.is_anti {
            color != self.my_color
        } else {
            color == self.my_color
        };
        let (mut alpha, mut beta) = ab;
        let mut best_eval = if is_maxing { Score::MIN } else { Score::MAX };

        for pl_move in allowed_moves {
            let new_board = board.with_move(&pl_move, color);
            let eval = self.minimax(
                new_board,
                depth - 1,
                (alpha, beta),
                color.opposite(),
            );

            if is_maxing {
                best_eval = max_of(eval, best_eval);
                alpha = max_of(eval, alpha);
            } else {
                best_eval = min_of(eval, best_eval);
                beta = min_of(eval, beta);
            }
            if beta <= alpha {
                break;
            }
        }
        best_eval
    }

    fn their_move(&mut self) {
        let coord = Chan::read();
        match coord {
            CLIMove::Coord(coord) => {
                log!(self, "their move: {}", coord.to_ab());
                let pl_move = self
                    .allowed_moves
                    .iter()
                    .find(|(ti, _)| *ti == coord)
                    .expect("Not a possible move from opponent")
                    .clone();
                self.board.apply_move(&pl_move, self.current_color);
            }
            CLIMove::Pass => {}
            _ => panic!("Unexpected command received"),
        }
    }

    /// Print `pass` to stdout
    fn pass_move(&self) {
        if self.current_color == self.my_color {
            Chan::send(CLIMove::Pass);
        } else {
            Chan::read();
        }
    }

    /// Updates allowed tiles for placing discs
    fn update_allowed(&mut self) {
        self.allowed_moves = self.board.allowed_moves(self.current_color);
    }

    fn switch_player(&mut self) {
        self.current_color = self.current_color.opposite();
    }

    /// Check winner
    fn wincheck(&mut self) {
        if self.allowed_moves.len() > 0 {
            self.win_state = EndState::Unknown;
            return;
        }

        let mut nblack = 0;
        let mut nwhite = 0;
        let mut has_empty = false;
        for disc in self.board.0.iter() {
            match disc {
                Cell::Empty => {
                    has_empty = true;
                }
                Cell::White => nwhite += 1,
                Cell::Black => nblack += 1,
                _ => {}
            };
        }

        if nblack == 0 {
            self.win_state = if self.is_anti {
                EndState::BlackWon
            } else {
                EndState::WhiteWon
            };
        } else if nwhite == 0 {
            self.win_state = if self.is_anti {
                EndState::WhiteWon
            } else {
                EndState::BlackWon
            };
        } else if !has_empty || self.win_state == EndState::OnePass {
            if nblack >= 32 {
                self.win_state = if self.is_anti {
                    EndState::WhiteWon
                } else {
                    EndState::BlackWon
                };
            } else {
                self.win_state = if self.is_anti {
                    EndState::BlackWon
                } else {
                    EndState::WhiteWon
                };
            }
        } else {
            self.win_state = EndState::OnePass;
        }
    }

    pub fn report(&self) {
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
    }
}
