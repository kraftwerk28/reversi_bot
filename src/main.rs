#[macro_use]
mod utils;
mod bot;
mod mcts;
mod mcts2;
mod minimax;
mod runner;

use runner::Runner;
use std::{thread, time};
use utils::{parse_args, select_bot_impl};

fn main() {
    let matches = parse_args();

    // Get current verstion (only for CI)
    if matches.is_present("version") {
        println!(env!("CARGO_PKG_VERSION"));
        return;
    }

    let bot = select_bot_impl(&matches);
    let mut runner = Runner::new(bot);

    runner.run();
    // bot.report();

    // Required to satisfy tester
    // But the tester still doesn't kill the process :(
    loop {
        thread::sleep(time::Duration::from_millis(100));
    }
}
