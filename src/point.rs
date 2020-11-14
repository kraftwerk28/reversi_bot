use crate::utils::*;
use std::{char, fmt};

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

#[test]
fn mirror1() {
    let p = Point::from_xy(6, 5);
    assert_eq!(p.unmirror4(), Point::from_xy(1, 2));
    assert_eq!(p.unmirror8(), Point::from_xy(2, 1));
}
