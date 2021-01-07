use crate::{board::Board, bot::Bot, utils::*};
use crossbeam::channel::{select, unbounded};
use rand::thread_rng;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    io::Write,
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

impl MCTSBot {
    pub fn new(arg_matches: &clap::ArgMatches) -> Self {
        let black_hole = read_black_hole(arg_matches);
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

        log!(bot, "alg: Basic MCTS");
        log!(bot, "black hole: {:?}", black_hole.map(|p| p.to_ab()));
        log!(bot, "my color: {:?}", my_color);
        log!(bot, "anti reversi mode: {}", is_anti);
        log!(bot, "move timeout: {}\n\nBEGIN:", move_maxtime);

        bot
    }

    fn mcts(&self) -> PlayerMove {
        if self.allowed_moves.len() == 1 {
            return self.allowed_moves.first().unwrap().clone();
        }
        let (results_tx, results_rx) = unbounded::<(EndState, usize)>();
        let (stop_tx, stop_rx) = unbounded::<()>();

        let results_handle = thread::spawn({
            let maxtime = self.move_maxtime;
            let my_color = self.my_color;
            let mut results: HashMap<usize, (u64, u64)> = HashMap::new();

            move || {
                let t = Instant::now();
                while t.elapsed() < maxtime {
                    if let Ok((end_state, move_index)) = results_rx.recv() {
                        let tup = results.entry(move_index).or_insert((0, 0));
                        if end_state.won(my_color) {
                            *tup = (tup.0 + 1, tup.1 + 1);
                        } else {
                            *tup = (tup.0, tup.1 + 1);
                        }
                    }
                }
                stop_tx.send(()).unwrap();
                results
            }
        });

        self.allowed_moves.par_iter().enumerate().for_each(
            move |(index, pl_move)| {
                let rng = thread_rng();
                loop {
                    let sim_result = Board::sim(
                        &self.board,
                        pl_move.clone(),
                        self.my_color,
                        self.is_anti,
                        rng,
                    );
                    select! {
                        recv(stop_rx) -> _ => break,
                        send(results_tx, (sim_result, index)) -> _ => {},
                    }
                }
            },
        );

        let results = results_handle.join().unwrap();
        log!(
            self,
            "total plays: {}; allowed moves: {}",
            results.values().map(|t| t.1).sum::<u64>(),
            self.allowed_moves.len(),
        );
        // log!(
        //     self,
        //     "{}",
        //     results
        //         .values()
        //         .map(|(wins, total)| format!("{}/{}", wins, total))
        //         .collect::<Vec<String>>()
        //         .join(", ")
        // );

        let mut best_move = &self.allowed_moves[0];
        let mut max_ratio = 0f64;
        for (index, (wins, total)) in results.iter() {
            let ratio = *wins as f64 / *total as f64;
            if ratio > max_ratio {
                best_move = &self.allowed_moves[*index];
                max_ratio = ratio;
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
