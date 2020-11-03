mod bot;
mod game;
mod utils;

use bot::*;
use clap::{App, AppSettings, Arg};
use std::str;
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
        .arg(
            Arg::with_name("max_depth")
                .long("depth")
                .takes_value(true)
                .default_value("8")
                .help("Maximum tree depth"),
        )
        .arg(
            Arg::with_name("log_file")
                .long("log")
                .takes_value(true)
                .help("File for logging"),
        )
        .arg(
            Arg::with_name("no-anti")
                .long("no-anti")
                .help("Play regular reversi"),
        )
        .get_matches();

    if matches.is_present("version") {
        println!(env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut bot = Bot::new(&matches);
    bot.run();
    println!(
        "{}",
        match bot.win_state {
            Tie => "Tie!",
            BlackWon => "Black won!",
            WhiteWon => "White won!",
            _ => "Game hadn't been completed.",
        }
    );
}
