mod game;
mod utils;

use clap::{App, AppSettings, Arg};
use game::*;
use utils::EndState::*;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("version")
                .short("v")
                .long("version")
                .help("Show version"),
        )
        .get_matches();

    if matches.is_present("version") {
        println!(env!("CARGO_PKG_VERSION"));
        return;
    }

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
