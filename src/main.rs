mod board;
mod bot;
mod tests;
mod utils;

use bot::*;
use clap::{App, AppSettings, Arg};

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
    bot.report();

    // Required to satisfy tester
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
