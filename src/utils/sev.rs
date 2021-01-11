#![allow(dead_code)]

use super::{
    board::{MainLine, MAINLINES},
    *,
};

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

#[rustfmt::skip]
const TILE_HEURISTICS: [[i32; 8]; 8] = [
    [ 410,  23,  13,   8,   8,  13,  23, 410 ],
    [  23, -75, -22, -51, -51, -22, -75,  23 ],
    [  13, -22,  41,   3,   3,  41, -22,  13 ],
    [   8, -51,   3, -87, -87,   3, -51,   8 ],
    [   8, -51,   3, -87, -87,   3, -51,   8 ],
    [  13, -22,  41,   3,   3,  41, -22,  13 ],
    [  23, -75, -22, -51, -51, -22, -75,  23 ],
    [ 410,  23,  13,   8,   8,  13,  23, 410 ],
];

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

// 410,  23,  13,   8
//      -75, -22, -51
//            41,   3
//                -87

#[inline]
pub fn tile_cost_2(pos: Point) -> Score {
    let i = pos.unmirror8().to_idx();

    match i {
        0 => 410,
        1 => 23,
        2 => 13,
        3 => 8,

        9 => -75,
        10 => -22,
        11 => -51,

        18 => 41,
        19 => 3,

        27 => -87,

        _ => unreachable!(),
    }
    // match i {
    //     0 => 99,
    //     1 => -8,
    //     2 => 8,
    //     3 => 6,

    //     9 => -24,
    //     10 => -4,
    //     11 => -3,

    //     18 => 7,
    //     19 => 4,

    //     27 => 0,

    //     _ => unreachable!(),
    // }
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

#[inline]
pub fn sev3(board: &Board, color: Cell, is_depth_even: bool) -> Score {
    let mut count = 0;

    for i in 0..64 {
        let p = Point::from_idx(i);
        let (x, y) = p.to_xy();
        let mut a = board.nempty_neighbours(p);
        if a == 0 {
            a = 6;
        }

        let heu = TILE_HEURISTICS[y as usize][x as usize] + (8 * a);
        let tile = board.at(p);

        if tile.is_empty() {
            if is_depth_even {
                count -= heu / 2;
            } else {
                count += heu / 2;
            }
        } else if tile == color {
            count += heu;
        } else {
            count -= heu;
        }
    }
    count += MAINLINES
        .iter()
        .map(|&ml| mainlines_penalty(board, ml, color))
        .sum::<i32>();
    count
}

#[inline]
pub fn mainlines_penalty(board: &Board, ml: MainLine, my_color: Cell) -> i32 {
    let penalty = 86;
    let line = board.mainline(ml);
    let size = 8;
    let mut count = 0;

    if line[1] == my_color && line[0].is_empty() {
        let mut i = 1;
        loop {
            i += 1;
            if i == size - 2 || line[i] != my_color {
                break;
            }
        }
        if i != size - 2 {
            count += penalty;
            if line[i + 1] == my_color {
                count += penalty;
            }
        }
    }
    // else if line[1] == opponent_color && line[0].is_empty() {
    //     let mut i = 1;
    //     loop {
    //         i += 1;
    //         if i == size - 2 || line[i] == opponent_color {
    //             break;
    //         }
    //     }
    //     if i != size - 2 {
    //         count += penalty;
    //         if line[i + 1] == opponent_color {
    //             count += penalty;
    //         }
    //     }
    // }

    if line[size - 2] == my_color && line[size - 1].is_empty() {
        let mut i = 1;
        loop {
            i += 1;
            if i == size - 2 || line[size - 1 - i] != my_color {
                break;
            }
        }
        if i != size - 2 {
            count += penalty;
            if line[size - 2 - i] == my_color {
                count += penalty;
            }
        }
    }
    // else if line[size - 2] == opponent_color && line[size - 1].is_empty() {
    //     let mut i = 1;
    //     loop {
    //         i += 1;
    //         if i == size - 2 || line[size - 1 - i] == opponent_color {
    //             break;
    //         }
    //     }
    //     if i != size - 2 {
    //         count += penalty;
    //         if line[size - 2 - i] == opponent_color {
    //             count += penalty;
    //         }
    //     }
    // }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_1() {
        let mut b = Board::initial(Some(Point::from_xy(1, 1)));
        b.place(Point::from_xy(0, 0), Cell::Black)
            .place(Point::from_xy(6, 0), Cell::Black);
        assert_eq!(
            static_eval_with_weights_1(&b, Cell::Black),
            BAD_EDGE + GOOD_CORNER
        );
    }
}
