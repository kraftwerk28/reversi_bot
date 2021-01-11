use crate::utils::{AllowedMoves, Cell, EndState, LogFile, PlayerMove};

pub trait Bot {
    fn allowed_tiles(&self) -> AllowedMoves;
    fn status(&self) -> EndState;
    fn apply_move(&mut self, player_move: &PlayerMove);
    fn current_color(&self) -> Cell;
    fn self_color(&self) -> Cell;
    fn set_color(&mut self, color: Cell);
    fn run_ai(&self) -> PlayerMove;
    fn get_logfile(&self) -> LogFile;
}
