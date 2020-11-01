use rand::{thread_rng, Rng};
use std::{char, collections::HashMap, io::stdin, process};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Black,
    White,
    BlackHole,
}

#[derive(Clone, Copy)]
pub enum EndState {
    Unknown,
    WhiteWon,
    BlackWon,
    Tie,
}

pub fn create_black_hole() -> usize {
    let mut rng = thread_rng();
    if rng.gen_bool(0.5) {
        rng.gen_range(0, 8 * 3 + 4 - 1)
    } else {
        rng.gen_range(8 * 4 + 5, 64)
    }
}

pub type TileIdx = i8;
pub type PlayerMove = (TileIdx, Vec<TileIdx>);
pub type Point = (TileIdx, TileIdx);
pub type AllowedMoves = Vec<PlayerMove>;

pub fn i2p(i: TileIdx) -> Point {
    (i % 8, i / 8)
}

pub fn p2i(p: Point) -> TileIdx {
    p.1 * 8 + p.0
}

pub fn p2ab(p: Point) -> String {
    format!(
        "{}{}",
        char::from_u32(p.0 as u32 + 65).unwrap(),
        (p.1 + 1).to_string()
    )
}

pub fn opposize_color(color: Cell) -> Cell {
    match color {
        Cell::White => Cell::Black,
        Cell::Black => Cell::White,
        _ => panic!("Wrong color"),
    }
}

pub fn input() -> String {
    loop {
        let mut sbuf = String::new();
        let bytes_read = stdin().read_line(&mut sbuf).unwrap();
        if bytes_read == 0 {
            process::exit(0);
        }
        let result = sbuf.trim().to_string();
        if result.len() > 0 {
            return result;
        }
    }
}

pub struct Chan {}

impl Chan {
    pub fn read_coord() -> Point {
        let s = input();
        let mut chars = s.chars();
        let x = chars.next().unwrap() as TileIdx;
        let y = chars.next().and_then(|c| c.to_digit(10)).unwrap() as TileIdx;
        (x - 65, y - 1)
    }

    pub fn read_color() -> Cell {
        match input().to_lowercase().as_str() {
            "black" => Cell::Black,
            "white" => Cell::White,
            _ => panic!("Bad color supplied"),
        }
    }

    pub fn send_coord(p: Point) {
        println!("{}", p2ab(p));
    }
}

pub const TRAVERSE_DIRECTIONS: [(TileIdx, TileIdx); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

pub fn repr_board(board: &[Cell]) -> String {
    let mut col = 0;
    let mut cells: Vec<Vec<&str>> = Vec::with_capacity(8);
    cells.push(Vec::with_capacity(8));
    for cell in board.iter() {
        let cell_repr = match cell {
            Cell::White => "██",
            Cell::Black => "░░",
            Cell::BlackHole => "BH",
            Cell::Empty => "▒▒",
        };
        cells.last_mut().unwrap().push(cell_repr);
        col += 1;
        if col == 8 {
            cells.push(Vec::with_capacity(8));
            col = 0;
        }
    }
    cells
        .iter()
        .map(|row| row.join(""))
        .collect::<Vec<String>>()
        .join("\n")
}
