use crate::{
    bot::Bot,
    utils::*,
    utils::{board::Board, tree::Node},
};
use crossbeam::channel;
use rayon::prelude::*;
use std::{
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
    is_anti: bool,
    exploitation_value: f64,
}

impl MCTSBot {
    pub fn new(arg_matches: &clap::ArgMatches) -> Self {
        let black_hole = read_black_hole(arg_matches);
        let my_color = Chan::read().color();

        let is_anti = !arg_matches.is_present("no_anti");

        let board = Board::initial(black_hole);
        let current_color = Cell::Black;

        let move_maxtime = arg_matches
            .value_of("time_limit")
            .map(|it| it.parse::<u64>().unwrap())
            .unwrap();

        let exploitation_value = arg_matches
            .value_of("exploitation_value")
            .map(str::parse::<f64>)
            .map(Result::unwrap)
            .unwrap_or(2f64.sqrt());

        let bot = Self {
            board,
            my_color,
            current_color,
            win_state: EndState::Unknown,

            log_file: get_logfile(&arg_matches),
            is_anti,
            move_maxtime: Duration::from_millis(move_maxtime),
            exploitation_value,
        };

        log!(bot, "alg: Advanced MCTS");
        log!(bot, "black hole: {:?}", black_hole.map(|p| p.to_ab()));
        log!(bot, "my color: {:?}", my_color);
        log!(bot, "anti reversi mode: {}", is_anti);
        log!(bot, "move timeout: {}\n\nBEGIN:", move_maxtime);

        bot
    }

    fn mcts(&self) -> PlayerMove {
        let allowed_moves = self.board.allowed_moves(self.current_color);

        if allowed_moves.len() == 1 {
            return allowed_moves.first().unwrap().clone();
        }

        let (stop_tx, stop_rx) = channel::unbounded::<()>();

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

                while let Err(_) = stop_rx.try_recv() {
                    let selected =
                        Node::selection(tree.clone(), self.exploitation_value);
                    let expanded = Node::expansion(selected);
                    let rollout_result =
                        expanded.borrow().simulate(self.is_anti);
                    Node::back_propagate(expanded, rollout_result);
                }

                let node = tree.borrow();
                (
                    (node.nwins, node.nvisits),
                    node.player_move.clone().unwrap(),
                )
            })
            .collect::<Vec<_>>();

        tim_thread.join().unwrap();

        log!(
            self,
            "final scores: [{}]",
            scores
                .iter()
                .map(|((w, v), _)| format!("{}/{}", w, v))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let mut max_score = f64::MIN;
        let mut best_move = &scores[0].1;
        for ((w, v), player_move) in scores.iter() {
            let score = *w as f64 / *v as f64;
            if score > max_score {
                max_score = score;
                best_move = &player_move
            }
        }
        best_move.clone()
    }
}

impl Bot for MCTSBot {
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
        self.mcts()
    }
}
