use crate::{board::Board, point::Point};
use clap::{App, AppSettings, Arg, ArgMatches};
use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::{stdin, stdout, BufWriter, Write},
    process,
    sync::Mutex,
};

// A small logging util
macro_rules! log {
    ($slf:ident, $($fmtargs:expr),+ $(,)*) => {
        if let Some(log_file) = &$slf.log_file {
            let lck = log_file.lock().unwrap();
            let mut writable = lck.borrow_mut();
            writeln!(writable, $($fmtargs),+).unwrap();
        }
    }
}

pub trait Bot {
    fn run(&mut self);
    fn report(&self);
}

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

    #[allow(dead_code)]
    pub fn is_disc(&self) -> bool {
        match self {
            Cell::White | Cell::Black => true,
            _ => false,
        }
    }
}

impl std::ops::Not for Cell {
    type Output = Self;
    fn not(self) -> Self {
        self.opposite()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum EndState {
    Unknown,
    WhiteWon,
    BlackWon,
    Tie,
}

impl EndState {
    pub fn is_over(&self) -> bool {
        match self {
            EndState::Unknown => false,
            _ => true,
        }
    }

    pub fn won(&self, cell: Cell) -> bool {
        match self {
            EndState::WhiteWon => cell == Cell::White,
            EndState::BlackWon => cell == Cell::Black,
            _ => false,
        }
    }
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

#[inline]
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

pub fn parse_args() -> ArgMatches<'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("version")
                .short("v")
                .long("version")
                .help("Show version"),
        )
        .arg(
            Arg::with_name("max_depth")
                .long("max-depth")
                .takes_value(true)
                .default_value("4")
                .env("MAX_DEPTH")
                .help("Maximum tree depth (only for minimax)"),
        )
        .arg(
            Arg::with_name("log_file")
                .long("log")
                .env("LOG")
                .takes_value(true)
                .help("File for logging"),
        )
        .arg(
            Arg::with_name("no_anti")
                .long("no-anti")
                .env("NO_ANTI")
                .help("Play regular reversi"),
        )
        .arg(
            Arg::with_name("time_limit")
                .long("time-limit")
                .short("t")
                .takes_value(true)
                .env("MAX_TIME")
                .default_value("4000")
                .help("Set time limit in milliseconds (for MCTS)"),
        )
        .arg(
            Arg::with_name("bot_impl")
                .long("bot-impl")
                .takes_value(true)
                .possible_values(&["minimax", "mcts_basic", "mcts"])
                .env("BOT_IMPL")
                .default_value("mcts"),
        )
        .arg(
            Arg::with_name("exploitation_value")
                .long("mcts-exp")
                .takes_value(true)
                .env("EXP"),
        )
        .get_matches()
}

pub type LogFile = Option<Mutex<RefCell<BufWriter<File>>>>;
pub fn get_logfile(matches: &ArgMatches) -> LogFile {
    matches.value_of("log_file").map(|name| {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(name)
            .map(|f| Mutex::new(RefCell::new(BufWriter::new(f))))
            .unwrap()
    })
}

#[inline]
pub fn wincheck(
    board: &Board,
    allowed_moves: &AllowedMoves,
    is_anti: bool,
    color: Cell,
) -> EndState {
    if allowed_moves.len() > 0 {
        return EndState::Unknown;
    }
    let maybepassmoves = board.allowed_moves(color.opposite());
    if maybepassmoves.len() > 0 {
        return EndState::Unknown;
    }

    let mut nblack = 0;
    let mut nwhite = 0;
    for disc in board.0.iter() {
        match disc {
            Cell::White => nwhite += 1,
            Cell::Black => nblack += 1,
            _ => {}
        };
    }

    if nblack > nwhite {
        if is_anti {
            EndState::WhiteWon
        } else {
            EndState::BlackWon
        }
    } else if nblack < nwhite {
        if is_anti {
            EndState::BlackWon
        } else {
            EndState::WhiteWon
        }
    } else {
        EndState::Tie
    }
}

pub fn uct_score(parent_nvisits: u64, nwins: u64, nvisits: u64, c: f64) -> f64 {
    if nvisits == 0 {
        f64::MAX
    } else {
        let xi = nwins as f64 / nvisits as f64;
        xi + c * ((parent_nvisits as f64).ln() / nvisits as f64).sqrt()
    }
}

pub fn select_bot_impl(matches: &ArgMatches) -> Box<dyn Bot> {
    use crate::{mcts, mcts2, minimax};
    match matches.value_of("bot_impl").unwrap() {
        "minimax" => Box::new(minimax::MinimaxBot::new(matches)),
        "mcts_basic" => Box::new(mcts::MCTSBot::new(matches)),
        "mcts" => Box::new(mcts2::MCTSBot::new(matches)),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod utils_test {
    use crate::{board::Board, utils::*};
    use std::convert::TryFrom;

    #[test]
    fn wincheck_1() {
        let s = "BBBBBBBB
             BBBBBBBB
             BBBBBBBB
             BBBBBBBB
             BBBBBBBB
             BBBBBBBB
             BBBBBBBB
             BBBBBBBB";
        let b = Board::try_from(s.to_string()).unwrap();
        assert_eq!(
            wincheck(&b, &b.allowed_moves(Cell::White), true, Cell::Black),
            EndState::WhiteWon
        );
    }

    #[test]
    fn wincheck_2() {
        let s = "BBBBBBBB
             BBBBBBBB
             BBBBB___
             BBBBB__W
             BBBBBB__
             BBBBBB__
             BBBBBBBB
             BBBBBBBB";
        let b = Board::try_from(s.to_string()).unwrap();
        let win =
            wincheck(&b, &b.allowed_moves(Cell::White), true, Cell::Black);
        assert!(win.is_over());
        assert_eq!(win, EndState::WhiteWon);
    }

    #[test]
    fn wincheck_3() {
        let s = "BBBBBBBB
             BBBBBBBB
             BBBBBBBB
             BBBBBBBB
             WWWWWWWW
             WWWWWWWW
             WWWWWWWW
             WWWWWWWW";
        let b = Board::try_from(s.to_string()).unwrap();
        let win =
            wincheck(&b, &b.allowed_moves(Cell::White), true, Cell::Black);
        assert!(win.is_over());
        assert_eq!(win, EndState::Tie);
    }
}
