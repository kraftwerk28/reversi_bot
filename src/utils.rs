use std::{char, io::stdin, process};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Black,
    White,
    BlackHole,
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
pub type PlayerMove = (TileIdx, Vec<TileIdx>);
pub type Point = (TileIdx, TileIdx);
pub type AllowedMoves = Vec<PlayerMove>;
pub type Score = usize;
pub type AlphaBeta = (Score, Score);

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

pub fn opposite_color(color: Cell) -> Cell {
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
    pub fn read() -> CLIMove {
        let s = input();
        match s.as_str() {
            "pass" => CLIMove::Pass,
            "black" => CLIMove::Color(Cell::Black),
            "white" => CLIMove::Color(Cell::White),
            _ => {
                let mut chars = s.chars();
                let x = chars.next().unwrap() as TileIdx;
                let y = chars.next().and_then(|c| c.to_digit(10)).unwrap()
                    as TileIdx;
                assert!((1..=8).contains(&y), "Bad coordinate supplied");
                assert!((65..=72).contains(&x), "Bad coordinate supplied");
                CLIMove::Coord((x - 65, y - 1))
            }
        }
    }

    pub fn send(p: CLIMove) {
        match p {
            CLIMove::Pass => {
                println!("pass");
            }
            CLIMove::Coord(p) => {
                println!("{}", p2ab(p));
            }
            _ => panic!("Unexpected command"),
        }
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

pub fn get_allowed_moves(board: &[Cell], color: Cell) -> AllowedMoves {
    let mut res: AllowedMoves = Vec::new();
    let rev_color = opposite_color(color);
    let only_current_tiles =
        board.iter().enumerate().filter(|(_, &cell)| cell == color);

    for (index, _) in only_current_tiles {
        let (x, y) = i2p(index as TileIdx);

        for (dx, dy) in TRAVERSE_DIRECTIONS.iter() {
            let (mut x, mut y) = (x + dx, y + dy);
            let mut to_be_flipped: Vec<TileIdx> = Vec::with_capacity(6);
            while (0..8).contains(&x) && (0..8).contains(&y) {
                let tile_index = p2i((x, y));
                let tile = board[p2i((x, y)) as usize];
                if tile == rev_color {
                    to_be_flipped.push(tile_index);
                } else {
                    if tile == Cell::Empty && to_be_flipped.len() > 0 {
                        let old_row =
                            res.iter_mut().find(|(t_i, _)| *t_i == tile_index);
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
