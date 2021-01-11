use crate::utils::*;
use point::Point;
use rand::{prelude::*, rngs::ThreadRng, Rng};
use std::fmt;

#[derive(Copy, Clone)]
pub struct Board(pub [Cell; 64]);

#[derive(Copy, Clone)]
pub enum MainLine {
    Top,
    Left,
    Right,
    Bottom,
    TopLeftBottomRight,
    TopRightBottomLeft,
}

pub const MAINLINES: &[MainLine] = &[
    MainLine::Top,
    MainLine::Left,
    MainLine::Right,
    MainLine::Bottom,
    MainLine::TopLeftBottomRight,
    MainLine::TopRightBottomLeft,
];

impl Board {
    pub fn initial(black_hole: Option<Point>) -> Self {
        let mut board = Board([Cell::Empty; 64]);
        board
            .place(Point::from_xy(3, 3), Cell::White)
            .place(Point::from_xy(4, 4), Cell::White)
            .place(Point::from_xy(3, 4), Cell::Black)
            .place(Point::from_xy(4, 3), Cell::Black);
        if let Some(bh) = black_hole {
            board.place(bh, Cell::BlackHole);
        }
        board
    }

    pub fn place(&mut self, p: Point, color: Cell) -> &mut Self {
        self.0[p.usize()] = color;
        self
    }

    #[inline]
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

    #[inline]
    pub fn allowed_moves(&self, color: Cell) -> AllowedMoves {
        get_allowed_moves(&self, color)
    }

    pub fn at(&self, point: Point) -> Cell {
        self.0[point.usize()]
    }

    pub fn count(&self, color: Cell) -> usize {
        self.0.iter().filter(|&&c| c == color).count()
    }

    #[inline]
    pub fn with_move(&self, player_move: &PlayerMove, color: Cell) -> Self {
        let mut result = self.clone();
        result.apply_move(player_move, color);
        result
    }

    #[inline]
    pub fn sim(
        board: &Board,
        player_move: PlayerMove,
        mut color: Cell,
        is_anti: bool,
        mut rng: ThreadRng,
    ) -> EndState {
        let mut new_board = board.with_move(&player_move, color);
        loop {
            color = color.opposite();
            let mut allowed = new_board.allowed_moves(color);
            let win = wincheck(&new_board, &allowed, is_anti, color);
            if win.is_over() {
                return win;
            }
            if allowed.len() == 0 {
                color = color.opposite();
                allowed = new_board.allowed_moves(color);
            }
            let mv = allowed[rng.gen_range(0, allowed.len())].clone();
            new_board.apply_move(&mv, color);
        }
    }

    #[inline]
    pub fn simauto(
        mut board: Board,
        mut color: Cell,
        is_anti: bool,
    ) -> EndState {
        let mut rng = thread_rng();
        loop {
            let mut allowed = board.allowed_moves(color);
            if allowed.len() == 1 {
                board.apply_move(&allowed.first().unwrap(), color);
                color = color.opposite();
                continue;
            }
            let win = wincheck(&board, &allowed, is_anti, color);
            if win.is_over() {
                return win;
            }
            if allowed.is_empty() {
                color = color.opposite();
                allowed = board.allowed_moves(color);
            }
            let mv = allowed[rng.gen_range(0, allowed.len())].clone();
            board.apply_move(&mv, color);
            color = color.opposite();
        }
    }

    #[inline]
    pub fn nempty_neighbours(&self, pos: Point) -> i32 {
        let (px, py) = pos.to_xy();
        let mut res = 0;
        for y in if py > 0 { -1 } else { 0 }..=if py < 7 { 1 } else { 0 } {
            for x in if px > 0 { -1 } else { 0 }..=if px < 7 { 1 } else { 0 } {
                if x == 0 && y == 0 {
                    continue;
                }
                if self.at(Point::from_xy(px + x, py + y)) == Cell::Empty {
                    res += 1;
                }
            }
        }
        res
    }

    #[inline]
    pub fn mainline(&self, mainline: MainLine) -> Vec<Cell> {
        use MainLine::*;
        let f = match mainline {
            Top => |i| (i, 0),
            Left => |i| (0, i),
            Right => |i| (7, i),
            Bottom => |i| (i, 7),
            TopLeftBottomRight => |i| (i, i),
            TopRightBottomLeft => |i| (7 - i, i),
        };
        (0..8)
            .map(f)
            .map(|(x, y)| self.at(Point::from_xy(x, y)))
            .collect()
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
        // let disccount = board.iter().filter(|d| d.is_disc()).count()
        Ok(Self(board))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_board_count() {
        let board = Board::initial(Some(Point::from_xy(0, 0)));
        assert_eq!(board.count(Cell::Black), 2);
        assert_eq!(board.count(Cell::White), 2);
    }

    #[test]
    fn test_board_count_move1() {
        let mut board = Board::initial(Some(Point::from_xy(0, 0)));
        let allowed_moves = board.allowed_moves(Cell::White);
        board.apply_move(allowed_moves.first().unwrap(), Cell::White);
        assert_eq!(board.count(Cell::Black), 1);
        assert_eq!(board.count(Cell::White), 4);
    }

    #[test]
    fn test_board_count_move2() {
        let board = Board::initial(Some(Point::from_xy(0, 0)));
        let allowed_moves = board.allowed_moves(Cell::Black);
        let board =
            board.with_move(allowed_moves.first().unwrap(), Cell::Black);
        assert_eq!(board.count(Cell::White), 1);
        assert_eq!(board.count(Cell::Black), 4);
    }

    #[test]
    fn test_board_indexes() {
        let b = Board::initial(Some(Point::from_xy(1, 1)));
        for (i, _) in b.0.iter().enumerate() {
            let tile_index = Point::from_idx(i as TileIdx).unmirror8().to_idx();
            assert!([0, 1, 2, 3, 9, 10, 11, 18, 19, 27].contains(&tile_index));
        }
    }

    #[test]
    fn test_empty_neighbours() {
        let board = Board::initial(Some(Point::from_xy(0, 0)));
        assert_eq!(board.nempty_neighbours(Point::from_xy(3, 4)), 5);
        assert_eq!(board.nempty_neighbours(Point::from_xy(5, 5)), 7);
    }

    #[test]
    fn test_board_mainline() {
        use Cell::*;
        let board = Board::initial(Some(Point::from_xy(0, 0)));
        assert_eq!(
            board.mainline(MainLine::TopLeftBottomRight),
            vec![BlackHole, Empty, Empty, White, White, Empty, Empty, Empty],
        );
        assert_eq!(
            board.mainline(MainLine::Top),
            vec![BlackHole, Empty, Empty, Empty, Empty, Empty, Empty, Empty],
        );
        assert_eq!(
            board.mainline(MainLine::Bottom),
            vec![Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty],
        );
        assert_eq!(
            board.mainline(MainLine::TopRightBottomLeft),
            vec![Empty, Empty, Empty, Black, Black, Empty, Empty, Empty],
        );
    }
}
