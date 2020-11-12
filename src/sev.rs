#![allow(dead_code)]
use crate::{board::Board, utils::*};

/// Simplest SEV, with depth of 5 leads to 0.89 winrate
pub fn static_eval(board: &Board, player_color: Cell) -> Score {
    let my_discs = board.count(player_color) as Score;
    let other_discs = board.count(player_color.opposite()) as Score;
    my_discs - other_discs
}

const GOOD_CORNER: Score = 10;
const GOOD_EDGE: Score = 5;
const BAD_EDGE: Score = -5;
const REGULAR: Score = 1;

#[inline]
fn tile_cost_1(pos: Point) -> Score {
    let (x, y) = pos.unmirror8().to_xy();
    if (x, y) == (0, 0) {
        GOOD_CORNER
    } else if x == 1 && (y == 0 || y == 1) {
        BAD_EDGE
    } else if y == 0 && (x == 2 || x == 3) {
        GOOD_EDGE
    } else if y == 1 && (x == 2 || x == 3) {
        BAD_EDGE
    } else {
        REGULAR
    }
}

#[inline]
fn tile_cost_2(pos: Point) -> Score {
    let i = pos.unmirror8().to_idx();
    match i {
        0 => 99,
        1 => -8,
        2 => 8,
        3 => 6,
        9 => -24,
        10 => -4,
        11 => -3,
        18 => 7,
        19 => 4,
        27 => 0,
        _ => unreachable!(),
    }
}

#[inline]
pub fn static_eval_with_weights_1(board: &Board, player_color: Cell) -> Score {
    let opposite = player_color.opposite();
    board
        .0
        .iter()
        .enumerate()
        .map(|(index, &t)| {
            let cost = tile_cost_1(Point::from_idx(index as i8));
            if t == player_color {
                cost
            } else if t == opposite {
                -cost
            } else {
                0
            }
        })
        .sum()
}

#[inline]
pub fn static_eval_with_weights_2(board: &Board, player_color: Cell) -> Score {
    let opposite = player_color.opposite();
    board
        .0
        .iter()
        .enumerate()
        .map(|(index, &t)| {
            let cost = tile_cost_2(Point::from_idx(index as i8));
            if t == player_color {
                cost
            } else if t == opposite {
                -cost
            } else {
                0
            }
        })
        .sum()
}

#[test]
fn test_score_1() {
    let mut b = Board::initial(Point::from_xy(1, 1));
    b.place(Point::from_xy(0, 0), Cell::Black)
        .place(Point::from_xy(6, 0), Cell::Black);
    assert_eq!(
        static_eval_with_weights_1(&b, Cell::Black),
        BAD_EDGE + GOOD_CORNER
    );
}
