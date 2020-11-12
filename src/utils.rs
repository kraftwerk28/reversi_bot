use crate::board::Board;
use std::{
    char, fmt,
    io::{stdin, stdout, Write},
    process,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Black,
    White,
    BlackHole,
}

impl Cell {
    pub fn opposite(&self) -> Cell {
        match self {
            Cell::White => Cell::Black,
            Cell::Black => Cell::White,
            _ => panic!("Unexpected color"),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Point(TileIdx);

impl Point {
    pub fn from_ab(ab: &str) -> Option<Self> {
        let mut chars = ab.chars();
        let x = chars.next().unwrap() as TileIdx;
        let y = chars.next().and_then(|c| c.to_digit(10)).unwrap() as TileIdx;
        if (1..=8).contains(&y) && (65..=72).contains(&x) {
            Some(Self::from_xy(x - 65, y - 1))
        } else {
            None
        }
    }

    pub fn to_ab(&self) -> String {
        let (x, y) = self.to_xy();
        format!(
            "{}{}",
            char::from_u32(x as u32 + 65).unwrap(),
            (y + 1).to_string()
        )
    }

    pub fn from_idx(idx: TileIdx) -> Self {
        Self(idx)
    }

    #[allow(dead_code)]
    pub fn to_idx(&self) -> TileIdx {
        self.0
    }

    pub fn from_xy(x: TileIdx, y: TileIdx) -> Self {
        Self(y * 8 + x)
    }

    pub fn to_xy(&self) -> (TileIdx, TileIdx) {
        (self.0 % 8, self.0 / 8)
    }

    pub fn usize(&self) -> usize {
        self.0 as usize
    }

    #[allow(dead_code)]
    pub fn mirror(&self) -> [Self; 4] {
        let (x, y) = self.to_xy();
        [
            Self::from_xy(x, y),
            Self::from_xy(7 - x, y),
            Self::from_xy(x, 7 - y),
            Self::from_xy(7 - x, 7 - y),
        ]
    }

    pub fn unmirror4(&self) -> Self {
        let (x, y) = self.to_xy();
        Self::from_xy(
            if x < 4 { x } else { 7 - x },
            if y < 4 { y } else { 7 - y },
        )
    }

    pub fn unmirror8(&self) -> Self {
        let (x, y) = self.unmirror4().to_xy();
        if x > y {
            Self::from_xy(x, y)
        } else {
            Self::from_xy(y, x)
        }
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (x, y) = self.to_xy();
        write!(f, "(x = {}, y = {})", x, y)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum EndState {
    Unknown,
    WhiteWon,
    BlackWon,
    Tie,
    OnePass,
}

pub enum CLIMove {
    Coord(Point),
    Color(Cell),
    Pass,
}

impl CLIMove {
    pub fn coord(self) -> Point {
        if let CLIMove::Coord(p) = self {
            p
        } else {
            panic!("Expected coordinate")
        }
    }
    pub fn color(self) -> Cell {
        if let CLIMove::Color(c) = self {
            c
        } else {
            panic!("Expected color")
        }
    }
}

pub type TileIdx = i8;
pub type PlayerMove = (Point, Vec<Point>);
pub type AllowedMoves = Vec<PlayerMove>;
pub type Score = i64;
pub type AlphaBeta = (Score, Score);

pub fn input() -> String {
    loop {
        let mut sbuf = String::new();
        let bytes_read = stdin().read_line(&mut sbuf).ok();
        if let Some(bytes_read) = bytes_read {
            if bytes_read == 0 {
                process::exit(0);
            }
            let result = sbuf.trim().to_string();
            if result.len() > 0 {
                return result;
            }
        } else {
            return "".to_string();
        }
    }
}

pub struct Chan {}

impl Chan {
    pub fn read() -> CLIMove {
        let s = input();
        match s.as_str() {
            "pass" => CLIMove::Pass,
            "black" => CLIMove::Color(Cell::Black),
            "white" => CLIMove::Color(Cell::White),
            _ => {
                if let Some(ab) = Point::from_ab(&s) {
                    CLIMove::Coord(ab)
                } else {
                    panic!("Unexpected command");
                }
            }
        }
    }

    pub fn send(p: CLIMove) {
        let line = match p {
            CLIMove::Pass => "pass".to_string(),
            CLIMove::Coord(p) => p.to_ab(),
            _ => panic!("Unexpected command"),
        };
        stdout()
            .write_all(format!("{}\n", line).as_bytes())
            .unwrap();
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
        if col == 8 && cells.len() < 8 {
            cells.push(Vec::with_capacity(8));
            col = 0;
        }
    }
    let letters = format!(
        "  {}",
        ('A'..='H')
            .map(|c| format!(" {}", c))
            .collect::<Vec<String>>()
            .join("")
    );
    let brd = cells
        .iter()
        .enumerate()
        .map(|(index, row)| format!(" {}{}", index + 1, row.join("")))
        .collect::<Vec<String>>()
        .join("\n");
    format!("{}\n{}\n", letters, brd)
}

pub fn max_of(s1: Score, s2: Score) -> Score {
    if s1 > s2 {
        s1
    } else {
        s2
    }
}

pub fn min_of(s1: Score, s2: Score) -> Score {
    if s1 > s2 {
        s2
    } else {
        s1
    }
}

pub fn get_allowed_moves(board: &Board, color: Cell) -> AllowedMoves {
    let mut res: AllowedMoves = Vec::new();
    let rev_color = color.opposite();
    let only_current_tiles = board
        .0
        .iter()
        .enumerate()
        .filter(|(_, &cell)| cell == color);

    for (index, _) in only_current_tiles {
        let start_point = Point::from_idx(index as TileIdx);
        let (x, y) = start_point.to_xy();

        for (dx, dy) in TRAVERSE_DIRECTIONS.iter() {
            let (mut x, mut y) = (x + dx, y + dy);
            let mut to_be_flipped: Vec<Point> = Vec::with_capacity(6);

            while (0..8).contains(&x) && (0..8).contains(&y) {
                let tile_pt = Point::from_xy(x, y);
                let tile = board.at(tile_pt);
                if tile == rev_color {
                    to_be_flipped.push(tile_pt);
                } else {
                    if tile == Cell::Empty && to_be_flipped.len() > 0 {
                        let old_row =
                            res.iter_mut().find(|(t_i, _)| *t_i == tile_pt);
                        if let Some(p_move) = old_row {
                            p_move.1.extend(to_be_flipped);
                        } else {
                            res.push((tile_pt, to_be_flipped));
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

#[test]
fn mirror1() {
    let p = Point::from_xy(6, 5);
    assert_eq!(p.unmirror4(), Point::from_xy(1, 2));
    assert_eq!(p.unmirror8(), Point::from_xy(2, 1));
}
