mod game;
mod utils;

use clap::App;
use game::*;
use utils::EndState::*;

fn main() {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    let mut game = GameState::new("bot.log");
    game.run();
    println!(
        "{}",
        match game.win_state {
            Tie => "Tie!",
            BlackWon => "Black won!",
            WhiteWon => "White won!",
            _ => "Game hadn't been completed.",
        }
    );
}
