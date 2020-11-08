use crate::{game::GameState, utils::*};
use clap::ArgMatches;
use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::Write,
    time::Instant,
};

// A small logging util
macro_rules! log {
    ($self:ident, $($fmtargs:expr),+) => {
        if let Some(log_file) = &$self.log_file {
            let mut writable = log_file.borrow_mut();
            writeln!(writable, $($fmtargs),+).unwrap();
        }
    }
}

pub struct Bot {
    pub game_state: GameState,
    pub win_state: EndState,
    pub my_color: Cell,
    pub current_color: Cell,
    max_tree_depth: usize,
    log_file: Option<RefCell<File>>,
    is_anti: bool,
}

impl Bot {
    pub fn new(arg_matches: &ArgMatches) -> Self {
        let black_hole = Chan::read_coord();
        let my_color = Chan::read_color();

        let log_file = arg_matches.value_of("log_file").map(|name| {
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(name)
                .map(|f| RefCell::new(f))
                .unwrap()
        });
        let max_tree_depth = arg_matches
            .value_of("max_depth")
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap();
        let is_anti = !arg_matches.is_present("no-anti");

        Self {
            game_state: GameState::new(black_hole),
            log_file,
            my_color,
            win_state: EndState::Unknown,
            current_color: Cell::Black,
            max_tree_depth,
            is_anti,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.wincheck();
            match self.win_state {
                EndState::Unknown | EndState::OnePass => {}
                _ => break,
            };
            if self.win_state == EndState::OnePass {
                log!(self, "Passing move");
            }
            if self.current_color == self.my_color {
                log!(self, "Making my move");
                self.my_move();
            } else {
                log!(self, "Waiting for their move");
                self.their_move();
            }
            self.switch_player();
            log!(self, "{}", repr_board(&self.game_state.board));
        }
    }

    fn my_move(&mut self) {
        let time = Instant::now();
        let coord = self.game_state.run_minimax(
            self.my_color,
            self.max_tree_depth,
            self.is_anti,
        );
        log!(
            self,
            "my move: {}; {}ms passed",
            p2ab(coord),
            time.elapsed().as_millis()
        );
        Chan::send_coord(coord);
    }

    fn their_move(&mut self) {
        let coord = Chan::read_coord();
        log!(self, "their move: {}", p2ab(coord));
        self.game_state.perform_move(coord, self.current_color);
    }

    fn wincheck(&mut self) {
        // TODO: handle case with only my circles!
        if self.game_state.allowed_moves.len() > 0 {
            self.win_state = EndState::Unknown;
            return;
        }
        let mut nblack = 0;
        let mut nwhite = 0;
        let mut has_empty = false;
        for disc in self.game_state.board.iter() {
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
            self.win_state = EndState::BlackWon;
        } else if nwhite == 0 {
            self.win_state = EndState::WhiteWon;
        } else if !has_empty || self.win_state == EndState::OnePass {
            if nblack >= 32 {
                self.win_state = EndState::WhiteWon;
            } else {
                self.win_state = EndState::BlackWon;
            }
        } else {
            self.win_state = EndState::OnePass;
            self.switch_player();
        }
    }

    fn switch_player(&mut self) {
        self.current_color = opposite_color(self.current_color);
        self.game_state.update_allowed(self.current_color);
    }
}

#[cfg(test)]
mod tests {
    use crate::{bot::Bot, game::*, utils::*};
    use std::convert::TryFrom;
    #[test]
    fn check_game_state_conds() {
        let board = "_BBBBBBB\
                     BBBBBBBB\
                     BBBBBBBB\
                     B_BWBBBB\
                     BBBBWBBB\
                     BBBBBBBB\
                     BBBBBBBB\
                     BBBBBBBB";
        let game_state = GameState::try_from(board.to_string()).unwrap();
        let mut bot = Bot {
            game_state,
            current_color: Cell::Black,
            win_state: EndState::Unknown,
            my_color: Cell::Black,
            log_file: None,
            max_tree_depth: 8,
            is_anti: true,
        };
    }
}
