#[macro_use]
mod utils;
mod board;
mod mcts;
mod minimax;
mod point;
mod sev;

use mcts::MCTSBot;
use minimax::MinimaxBot;
use std::{thread, time};
use utils::{parse_args, Bot};

fn main() {
    let matches = parse_args();

    // Get current verstion (only for CI)
    if matches.is_present("version") {
        println!(env!("CARGO_PKG_VERSION"));
        return;
    }

    // let mut bot = MinimaxBot::new(&matches);
    let mut bot = MCTSBot::new(&matches);
    bot.run();
    bot.report();

    // Required to satisfy tester
    // But the tester still doesn't kill the process :(
    loop {
        thread::sleep(time::Duration::from_millis(100));
    }
}
