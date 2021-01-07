use crate::utils::{AllowedMoves, Cell, EndState, PlayerMove};

pub trait Bot {
    // fn run(&mut self);
    fn allowed_tiles(&self) -> AllowedMoves;
    fn status(&self) -> EndState;
    fn apply_move(&mut self, player_move: &PlayerMove);
    fn current_color(&self) -> Cell;
    fn self_color(&self) -> Cell;
    fn set_color(&mut self, color: Cell);
    fn report(&self);
    fn run_ai(&self) -> PlayerMove;
}
