mod game;
mod utils;

use game::*;
use utils::EndState::*;

fn main() {
    let mut game = GameState::new("bot.log");
    game.run();
    println!(
        "{}",
        match game.win_state {
            Tie => "Tie!",
            BlackWon => "White won!",
            WhiteWon => "Black won!",
            _ => "Game hadn't been completed.",
        }
    );
}
