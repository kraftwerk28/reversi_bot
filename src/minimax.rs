use crate::{
    bot::Bot,
    utils::sev::*,
    utils::{board::Board, *},
};
use clap::ArgMatches;
use rayon::prelude::*;
use std::{io::Write, sync::Mutex};

pub struct MinimaxBot {
    board: Board,
    my_color: Cell,
    current_color: Cell,
    win_state: EndState,
    max_tree_depth: usize,
    log_file: LogFile,
    is_anti: bool,
}

impl MinimaxBot {
    pub fn new(arg_matches: &ArgMatches) -> Self {
        let black_hole = read_black_hole(arg_matches);
        let my_color = Chan::read().color();

        let is_anti = !arg_matches.is_present("no_anti");

        let board = Board::initial(black_hole);
        let current_color = Cell::Black;
        let _allowed_moves = board.allowed_moves(current_color);
        let max_tree_depth = arg_matches
            .value_of("max_depth")
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap();

        let bot = Self {
            board,
            my_color,
            current_color,
            win_state: EndState::Unknown,
            max_tree_depth,
            log_file: get_logfile(&arg_matches),
            is_anti,
        };

        log!(bot, "alg: MiniMax");
        log!(bot, "black hole: {:?}", black_hole.map(|p| p.to_ab()));
        log!(bot, "my color: {:?}", my_color);
        log!(bot, "anti reversi mode: {}", is_anti);
        log!(bot, "tree depth: {}\n\nBEGIN:", max_tree_depth);

        bot
    }

    #[inline]
    #[allow(dead_code)]
    fn run_minimax(&self) -> PlayerMove {
        let allowed_moves = self.board.allowed_moves(self.current_color);

        let opponent_color = self.my_color.opposite();
        let best_eval =
            Mutex::new((Score::MIN, allowed_moves.first().unwrap()));

        // let best_eval = {
        //     let score = if self.is_anti { Score::MAX } else { Score::MIN };
        //     let tup = (score, allowed_moves.first().unwrap());
        //     Mutex::new(tup)
        // };

        let alphabeta = (Score::MIN, Score::MAX);

        for pl_move in allowed_moves.iter() {
            let next_board = self.board.with_move(pl_move, self.my_color);
            let eval = self.minimax(
                next_board,
                self.max_tree_depth - 1,
                alphabeta,
                self.my_color.opposite(),
            );
        }

        allowed_moves.par_iter().for_each(|pl_move| {
            let next_board = self.board.with_move(pl_move, self.my_color);
            let score = self.minimax(
                next_board,
                self.max_tree_depth,
                alphabeta,
                opponent_color,
            );

            let mut lck = best_eval.lock().unwrap();
            if score > lck.0 {
                *lck = (score, pl_move);
            }
        });

        let best_eval = best_eval.lock().unwrap();
        best_eval.1.clone()
    }

    #[inline]
    #[allow(dead_code)]
    fn minimax(
        &self,
        board: Board,
        depth: usize,
        ab: AlphaBeta,
        color: Cell,
    ) -> Score {
        let allowed_moves = board.allowed_moves(color);

        if depth == 0 || allowed_moves.is_empty() {
            let mul = if self.my_color == color { 1 } else { -1 };
            // return sev3(&board, self.my_color) * mul;
            return static_eval_with_weights_2(&board, self.my_color) * mul;
        }

        let (mut alpha, beta) = ab;
        let mut best_eval = Score::MIN;

        for pl_move in allowed_moves {
            let eval = self.minimax(
                board.with_move(&pl_move, color),
                depth - 1,
                (-beta, -alpha),
                color.opposite(),
            );

            best_eval = max_of(best_eval, -1 * eval);
            alpha = max_of(alpha, best_eval);
            if alpha >= beta {
                break;
            }
        }
        best_eval
    }

    pub fn run_negamax(&self) -> PlayerMove {
        let allowed_moves = self.board.allowed_moves(self.current_color);
        let first = allowed_moves.first();
        if allowed_moves.len() == 1 {
            return first.unwrap().clone();
        }

        let mut best_move = (Score::MIN, first.unwrap());
        let (mut alpha, beta) = (Score::MIN, Score::MAX);
        let mul = if self.is_anti { 1 } else { -1 };

        for pl_move in allowed_moves.iter() {
            let new_board = self.board.with_move(&pl_move, self.my_color);
            let score = self.negamax(
                new_board,
                self.max_tree_depth,
                alpha,
                beta,
                !self.my_color,
            ) * mul;
            if score > best_move.0 {
                best_move = (score, &pl_move);
            }

            alpha = max_of(alpha, best_move.0);
            if alpha >= beta {
                break;
            }
        }
        best_move.1.clone()
    }

    #[inline]
    pub fn negamax(
        &self,
        board: Board,
        depth: usize,
        mut alpha: i32,
        beta: i32,
        color: Cell,
    ) -> Score {
        let allowed_moves = board.allowed_moves(color);
        if depth == 0 || allowed_moves.is_empty() {
            let mul = if color == self.my_color { 1 } else { -1 };
            let even_depth = self.max_tree_depth % 2 == 0;
            return sev3(&board, self.my_color, even_depth) * mul;
        }

        let mut best = Score::MIN;
        for pl_move in allowed_moves.iter() {
            let new_board = board.with_move(pl_move, color);
            let sc = -self.negamax(new_board, depth - 1, -beta, -alpha, !color);
            if sc > best {
                best = sc;
            }
            alpha = max_of(alpha, best);
            if alpha >= beta {
                break;
            }
        }

        best
    }
}

impl Bot for MinimaxBot {
    fn status(&self) -> EndState {
        self.win_state
    }
    fn allowed_tiles(&self) -> AllowedMoves {
        self.board.allowed_moves(self.current_color)
    }
    fn apply_move(&mut self, player_move: &PlayerMove) {
        self.board.apply_move(&player_move, self.current_color);
    }
    fn current_color(&self) -> Cell {
        self.current_color
    }
    fn set_color(&mut self, color: Cell) {
        self.current_color = color;
    }
    fn self_color(&self) -> Cell {
        self.my_color
    }
    fn run_ai(&self) -> PlayerMove {
        self.run_negamax()
    }
    fn get_logfile(&self) -> LogFile {
        self.log_file.clone()
    }
}
