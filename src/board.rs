use crate::utils::*;
use std::fmt;

#[derive(Copy, Clone)]
pub struct Board(pub [Cell; 64]);

impl Board {
    pub fn initial(black_hole: Point) -> Self {
        *Board([Cell::Empty; 64])
            .place(Point::from_xy(3, 3), Cell::White)
            .place(Point::from_xy(4, 4), Cell::White)
            .place(Point::from_xy(3, 4), Cell::Black)
            .place(Point::from_xy(4, 3), Cell::Black)
            .place(black_hole, Cell::BlackHole)
    }

    pub fn place(&mut self, p: Point, color: Cell) -> &mut Self {
        self.0[p.usize()] = color;
        self
    }

    pub fn apply_move(
        &mut self,
        player_move: &PlayerMove,
        color: Cell,
    ) -> &mut Self {
        self.0[player_move.0.usize()] = color;
        for i in player_move.1.iter() {
            self.0[i.usize()] = color;
        }
        self
    }

    pub fn allowed_moves(&self, color: Cell) -> AllowedMoves {
        get_allowed_moves(&self, color)
    }

    pub fn at(&self, point: Point) -> Cell {
        self.0[point.usize()]
    }

    pub fn count(&self, color: Cell) -> usize {
        self.0.iter().filter(|&&c| c == color).count()
    }

    pub fn with_move(&self, player_move: &PlayerMove, color: Cell) -> Self {
        let mut result = self.clone();
        result.apply_move(player_move, color);
        result
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", repr_board(&self.0))
    }
}

impl std::convert::TryFrom<String> for Board {
    type Error = String;

    fn try_from(board_str: String) -> Result<Self, Self::Error> {
        let mut board = [Cell::Empty; 64];
        let mut idx = 0;
        for ch in board_str.chars().filter(|ch| !ch.is_whitespace()) {
            let cell = match ch {
                'B' => Some(Cell::Black),
                'H' => Some(Cell::BlackHole),
                'W' => Some(Cell::White),
                '_' => Some(Cell::Empty),
                _ => None,
            };

            if let Some(cell) = cell {
                board[idx] = cell;
                idx += 1;
            } else {
                return Err(format!("Unexpected char inside board: {}", ch));
            }
        }
        Ok(Self(board))
    }
}

// impl IntoIterator for Board {
//     type Item = &'static Cell;
//     type IntoIter = std::slice::Iter<'static, Self::Item>;
//     fn into_iter(self) -> Self::IntoIter {
//         (&self.0).into_iter()
//     }
// }

#[test]
fn test_board_count() {
    let board = Board::initial(Point::from_xy(0, 0));
    assert_eq!(board.count(Cell::Black), 2);
    assert_eq!(board.count(Cell::White), 2);
}

#[test]
fn test_board_count_move1() {
    let mut board = Board::initial(Point::from_xy(0, 0));
    let allowed_moves = board.allowed_moves(Cell::White);
    board.apply_move(allowed_moves.first().unwrap(), Cell::White);
    assert_eq!(board.count(Cell::Black), 1);
    assert_eq!(board.count(Cell::White), 4);
}

#[test]
fn test_board_count_move2() {
    let board = Board::initial(Point::from_xy(0, 0));
    let allowed_moves = board.allowed_moves(Cell::Black);
    let board = board.with_move(allowed_moves.first().unwrap(), Cell::Black);
    assert_eq!(board.count(Cell::White), 1);
    assert_eq!(board.count(Cell::Black), 4);
}

#[test]
fn test_board_indexes() {
    let b = Board::initial(Point::from_xy(1, 1));
    for (i, c) in b.0.iter().enumerate() {
        let tile_index = Point::from_idx(i as TileIdx).unmirror8().to_idx();
        assert!([0, 1, 2, 3, 9, 10, 11, 18, 19, 27].contains(&tile_index));
    }
}
