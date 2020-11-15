use crate::{board::Board, sev::*, utils::*};
use clap::ArgMatches;
use rayon::prelude::*;
use std::{
    io::Write,
    sync::Mutex,
    time::{Duration, Instant},
};

pub struct MinimaxBot {
    board: Board,
    my_color: Cell,
    current_color: Cell,
    win_state: EndState,
    allowed_moves: AllowedMoves,
    max_tree_depth: usize,
    log_file: LogFile,
    is_anti: bool,
    total_timer: Duration,
}

impl MinimaxBot {
    pub fn new(arg_matches: &ArgMatches) -> Self {
        let black_hole = Chan::read().coord();
        let my_color = Chan::read().color();

        let is_anti = !arg_matches.is_present("no_anti")
            && !std::env::var("NO_ANTI")
                .map(|it| it == "1")
                .unwrap_or(false);

        let board = Board::initial(black_hole);
        let current_color = Cell::Black;
        let allowed_moves = board.allowed_moves(current_color);
        let max_tree_depth = get_tree_depth(&arg_matches);

        let bot = Self {
            board,
            my_color,
            current_color,
            win_state: EndState::Unknown,
            allowed_moves,
            max_tree_depth,
            log_file: get_logfile(&arg_matches),
            is_anti,
            total_timer: Duration::default(),
        };

        log!(bot, "black hole: {:?}", black_hole.to_ab());
        log!(bot, "my color: {:?}", my_color);
        log!(bot, "anti reversi mode: {}", is_anti);
        log!(bot, "tree depth: {}\n\nBEGIN:", max_tree_depth);

        bot
    }

    fn run_minimax(&self) -> PlayerMove {
        let best_eval = {
            let score = if self.is_anti { Score::MAX } else { Score::MIN };
            let tup = (score, self.allowed_moves.first().unwrap());
            Mutex::new(tup)
        };
        let alphabeta = (Score::MIN, Score::MAX);
        self.allowed_moves.par_iter().for_each(|pl_move| {
            let score = self.minimax(
                self.board.with_move(pl_move, self.my_color),
                self.max_tree_depth,
                alphabeta,
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
}

impl Bot for MinimaxBot {
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
                    let timer = Instant::now();
                    let pl_move = self.run_minimax();
                    log!(
                        self,
                        "my move: {}, time spent: {}ms",
                        pl_move.0.to_ab(),
                        timer.elapsed().as_millis(),
                    );
                    self.total_timer += timer.elapsed();
                    self.board.apply_move(&pl_move, self.current_color);
                    Chan::send(CLIMove::Coord(pl_move.0));
                } else {
                    let coord = Chan::read().coord();
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
}
