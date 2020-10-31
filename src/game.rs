use crate::utils::*;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
};

pub struct GameState {
    pub board: [Cell; 64],
    pub current_color: Cell,
    pub my_color: Cell,
    pub win_state: EndState,
    rng: ThreadRng,
    log_file: File,
}

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
            .write(true)
            .truncate(true)
            .open(log_file_name)
            .unwrap();
        writeln!(&log_file, "my color: {:?}", my_color).unwrap();

        GameState {
            board,
            current_color: Cell::Black,
            my_color,
            win_state: EndState::Unknown,
            rng: thread_rng(),
            log_file,
        }
    }

    fn get_allowed_moves(&self) -> AllowedMoves {
        let mut res: AllowedMoves = HashMap::new();
        let rev_color = opposize_color(self.current_color);
        let only_current_tiles = self
            .board
            .iter()
            .enumerate()
            .filter(|(index, &cell)| cell == self.current_color);
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
                            res.insert(tile_index, to_be_flipped);
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

    pub fn step(&mut self) {
        // TODO: implement minimax!
        if self.current_color == self.my_color {
            self.place_disc();
        } else {
            self.recv_disc();
        }
        writeln!(self.log_file, "{}", repr_board(&self.board)).unwrap();
        self.current_color = opposize_color(self.current_color);
    }

    fn place_disc(&mut self) {
        let allowed = self.get_allowed_moves();
        let rand_tile = self.rng.gen_range(0, allowed.len());
        let tile_index = *allowed.keys().collect::<Vec<&TileIdx>>()[rand_tile];
        let ab = p2ab(i2p(tile_index));
        println!("{}", ab);
        writeln!(self.log_file, "my move: {}", ab).unwrap();

        let flip_row = allowed
            .get(&tile_index)
            .expect("Not a possible move made by me.");
        self.board[tile_index as usize] = self.current_color;
        for &tile_index in flip_row.iter() {
            self.board[tile_index as usize] = self.current_color;
        }
    }

    fn recv_disc(&mut self) {
        let allowed = self.get_allowed_moves();
        let coord = Chan::read_coord();
        writeln!(self.log_file, "their move: {}", p2ab(coord)).unwrap();

        let flip_row = allowed
            .get(&p2i(coord))
            .expect("Not a possible move from opponent.");
        self.board[p2i(coord) as usize] = self.current_color;
        for &tile_index in flip_row.iter() {
            self.board[tile_index as usize] = self.current_color;
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.win_state {
                EndState::Unknown => self.step(),
                _ => break,
            }
        }
    }
}
