use crate::utils::*;
use rayon::prelude::*;
use std::{convert::TryFrom, sync::Mutex, usize};

pub struct GameState {
    pub board: [Cell; 64],
    pub allowed_moves: AllowedMoves,
    pub best_score: Score,
}

impl GameState {
    pub fn new(black_hole: Point) -> GameState {
        let board = [Cell::Empty; 64];
        let mut state = GameState {
            board,
            allowed_moves: Vec::new(),
            best_score: 0,
        };
        state
            .set_cell((3, 3), Cell::White)
            .set_cell((4, 4), Cell::White)
            .set_cell((3, 4), Cell::Black)
            .set_cell((4, 3), Cell::Black)
            .set_cell(black_hole, Cell::BlackHole);
        state.allowed_moves = get_allowed_moves(&state.board, Cell::Black);

        state
    }

    pub fn run_minimax(
        &mut self,
        color: Cell,
        max_depth: usize,
        is_anti: bool,
    ) -> Point {
        let best_score = {
            let best_score = if is_anti { Score::MAX } else { Score::MIN };
            let best = (self.allowed_moves.first().unwrap(), best_score);
            Mutex::new(best)
        };

        let alphabeta = (Score::MIN, Score::MAX);

        self.allowed_moves.par_iter().for_each(|player_move| {
            let score = self.minimax(
                color,
                player_move,
                max_depth,
                alphabeta,
                !is_anti,
            );
            let mut lck = best_score.lock().unwrap();
            let best_score = (*lck).1;
            if (score < best_score && is_anti)
                || (score > best_score && !is_anti)
            {
                *lck = (player_move, score);
            }
        });
        let (best_move, _) = *best_score.lock().unwrap();
        let best_move = best_move.clone();
        self.apply_move(&best_move, color);
        i2p(best_move.0)
    }

    fn minimax(
        &self,
        color: Cell,
        player_move: &PlayerMove,
        depth: usize,
        ab: AlphaBeta,
        is_maxing: bool,
    ) -> Score {
        if depth == 0 || self.allowed_moves.len() == 0 {
            return self.static_eval(color);
        }

        let next_state = self.copy_with_step(player_move, color);
        let (mut alpha, mut beta) = ab;
        let mut best_eval = if is_maxing { Score::MIN } else { Score::MAX };
        if is_maxing {
            for child_move in next_state.allowed_moves.iter() {
                let eval = next_state.minimax(
                    color,
                    child_move,
                    depth - 1,
                    (alpha, beta),
                    false,
                );
                best_eval = max_of(best_eval, eval);
                alpha = max_of(alpha, eval);
                if beta <= alpha {
                    break;
                }
            }
        } else {
            for child_move in next_state.allowed_moves.iter() {
                let eval = next_state.minimax(
                    color,
                    child_move,
                    depth - 1,
                    (alpha, beta),
                    true,
                );
                best_eval = min_of(best_eval, eval);
                beta = min_of(beta, eval);
                if beta <= alpha {
                    break;
                }
            }
        }
        best_eval
    }

    fn copy_with_step(&self, player_move: &PlayerMove, color: Cell) -> Self {
        let new_board = self.board.clone();
        let mut new_state = GameState {
            board: new_board,
            allowed_moves: Vec::new(),
            ..*self
        };
        new_state.apply_move(player_move, color);
        new_state.update_allowed(opposite_color(color));
        new_state
    }

    fn set_cell(&mut self, pos: Point, cell: Cell) -> &mut Self {
        self.board[p2i(pos) as usize] = cell;
        self
    }

    pub fn perform_move(&mut self, tile: Point, color: Cell) {
        let tile_index = p2i(tile);
        let player_move = self
            .allowed_moves
            .iter()
            .find(|(index, _)| *index == tile_index)
            .expect("Invalid move")
            .clone();
        self.apply_move(&player_move, color);
        self.update_allowed(opposite_color(color));
    }

    fn apply_move(&mut self, player_move: &PlayerMove, color: Cell) {
        self.board[player_move.0 as usize] = color;
        for &i in player_move.1.iter() {
            self.board[i as usize] = color;
        }
    }

    fn static_eval(&self, color: Cell) -> Score {
        self.board.iter().filter(|&cell| *cell == color).count()
    }

    pub fn update_allowed(&mut self, color: Cell) {
        self.allowed_moves = get_allowed_moves(&self.board, color);
    }
}

impl TryFrom<String> for GameState {
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
        Ok(Self {
            board,
            allowed_moves: get_allowed_moves(&board, Cell::Black),
            best_score: 0,
        })
    }
}
