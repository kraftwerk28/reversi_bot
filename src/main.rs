pub mod game;
pub mod utils;

use game::*;
use utils::*;

fn main() {
    let mut game = GameState::new("bot.log");
    game.run();
    println!(
        "{}",
        match game.win_state {
            EndState::Tie => "Tie!",
            EndState::BlackWon => "White won!",
            EndState::WhiteWon => "Black won!",
            _ => "An error occured",
        }
    );
}
