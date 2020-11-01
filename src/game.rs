use crate::utils::*;
use rayon::prelude::*;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    time::Instant,
};

// A small logging util
macro_rules! log {
    ($self:ident, $($fmtargs:expr),+) => {
        if let Some(log_file) = &mut $self.log_file {
            writeln!(log_file, $($fmtargs),+).unwrap();
        }
    }
}

pub struct GameState {
    pub board: [Cell; 64],
    pub current_color: Cell,
    pub my_color: Cell,
    pub win_state: EndState,
    log_file: Option<File>,
    allowed_moves: AllowedMoves,
    was_passed: bool,
}

const MM_MAXDEPTH: usize = 4;

impl GameState {
    pub fn new(log_file_name: &str) -> GameState {
        let black_hole = Chan::read_coord();
        let my_color = Chan::read_color();

        let mut board = [Cell::Empty; 64];
        board[p2i((3, 3)) as usize] = Cell::White;
        board[p2i((4, 4)) as usize] = Cell::White;
        board[p2i((3, 4)) as usize] = Cell::Black;
        board[p2i((4, 3)) as usize] = Cell::Black;
        board[p2i(black_hole) as usize] = Cell::BlackHole;

        let log_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(log_file_name)
            .unwrap();

        Self {
            board,
            current_color: Cell::Black,
            my_color,
            win_state: EndState::Unknown,
            log_file: Some(log_file),
            allowed_moves: Vec::new(),
            was_passed: false,
        }
    }

    fn get_allowed_moves(&self) -> AllowedMoves {
        // let mut res: AllowedMoves = HashMap::new();
        let mut res: AllowedMoves = Vec::new();
        let rev_color = opposize_color(self.current_color);
        let only_current_tiles = self
            .board
            .iter()
            .enumerate()
            .filter(|(_, &cell)| cell == self.current_color);
        for (index, _) in only_current_tiles {
            let (x, y) = i2p(index as TileIdx);

            for (dx, dy) in TRAVERSE_DIRECTIONS.iter() {
                let (mut x, mut y) = (x + dx, y + dy);
                let mut to_be_flipped: Vec<TileIdx> = Vec::with_capacity(6);
                while (0..8).contains(&x) && (0..8).contains(&y) {
                    let tile_index = p2i((x, y));
                    let tile = self.board[p2i((x, y)) as usize];
                    if tile == rev_color {
                        to_be_flipped.push(tile_index);
                    } else {
                        if tile == Cell::Empty && to_be_flipped.len() > 0 {
                            let old_row = res
                                .iter_mut()
                                .find(|(t_i, _)| *t_i == tile_index);
                            if let Some(p_move) = old_row {
                                p_move.1.extend(to_be_flipped);
                            } else {
                                res.push((tile_index, to_be_flipped));
                            }
                        }
                        break;
                    }
                    x += dx;
                    y += dy;
                }
            }
        }

        res
    }

    fn step(&mut self) {
        self.allowed_moves = self.get_allowed_moves();
        log!(self, "awailable tiles: {}", self.allowed_moves.len());

        if self.allowed_moves.len() == 0 {
            log!(self, "passing move");
            if self.was_passed {
                self.win_state = EndState::Tie;
                return;
            }
            self.current_color = opposize_color(self.current_color);
            self.was_passed = true;
            self.step();
            return;
        }

        if self.current_color == self.my_color {
            self.place_disc();
        } else {
            self.recv_disc();
        }
        log!(self, "{}", repr_board(&self.board));
        self.current_color = opposize_color(self.current_color);
    }

    fn place_disc(&mut self) {
        let now = Instant::now();
        let (tile_index, flip_row) = self.minimax();
        let ab = p2ab(i2p(tile_index));
        println!("{}", ab);
        log!(self, "my move: {}", ab);

        self.board[tile_index as usize] = self.current_color;
        for &tile_index in flip_row.iter() {
            self.board[tile_index as usize] = self.current_color;
        }
        log!(self, "step duration: {}ms", now.elapsed().as_millis());
    }

    fn recv_disc(&mut self) {
        let coord = Chan::read_coord();
        log!(self, "their move: {}", p2ab(coord));

        let tile_index = p2i(coord);
        let flip_row = self
            .allowed_moves
            .iter()
            .find(|(t_i, _)| *t_i == tile_index)
            .expect("Not a possible move from opponent.")
            .1
            .clone();
        self.board[p2i(coord) as usize] = self.current_color;
        for &tile_index in flip_row.iter() {
            self.board[tile_index as usize] = self.current_color;
        }
    }

    pub fn run(&mut self) {
        log!(self, "my color: {:?}", self.my_color);
        loop {
            match self.win_state {
                EndState::Unknown => self.step(),
                _ => break,
            }
        }
    }

    fn minimax(&mut self) -> PlayerMove {
        if self.allowed_moves.len() == 1 {
            let (tile, row) = self.allowed_moves.iter().next().unwrap();
            return (*tile, row.to_owned());
        }
        self.allowed_moves
            .par_iter()
            .max_by(|m1, m2| {
                let score1 = self.copy_with_step(m1).calc_mm_score(0, 0);
                let score2 = self.copy_with_step(m2).calc_mm_score(0, 0);
                score1.cmp(&score2)
            })
            .map(|(tile, row)| (*tile, row.clone()))
            .unwrap()
    }

    fn calc_mm_score(self, depth: usize, parent_score: usize) -> usize {
        if self.allowed_moves.len() == 0 {
            return 0;
        }
        let score = {
            let lens =
                self.allowed_moves.iter().map(|flip_row| flip_row.1.len());
            // Ex-change this things to make it regular reversi game
            if self.current_color == self.my_color {
                lens.min()
            } else {
                lens.max()
            }
            .unwrap()
        };
        if depth >= MM_MAXDEPTH {
            score
        } else {
            let sub_score: usize = self
                .allowed_moves
                .iter()
                .map(|t| {
                    self.copy_with_step(&t).calc_mm_score(depth + 1, score)
                })
                .sum();
            score + sub_score
        }
    }

    fn copy_with_step(&self, player_move: &PlayerMove) -> Self {
        let (tile_index, flip_row) = player_move;
        let mut new_board = self.board.clone();

        new_board[*tile_index as usize] = self.current_color;
        for &tile in flip_row {
            new_board[tile as usize] = self.current_color;
        }

        let mut dummy_state = Self {
            board: new_board,
            current_color: opposize_color(self.current_color),
            log_file: None,
            allowed_moves: Vec::new(),
            ..*self
        };
        dummy_state.allowed_moves = dummy_state.get_allowed_moves();
        dummy_state
    }
}
