use crate::{
    bot::Bot,
    utils::{CLIMove, Chan},
};

pub struct Runner {
    bot: Box<dyn Bot>,
}

impl Runner {
    pub fn new(bot: Box<dyn Bot>) -> Self {
        Self { bot }
    }

    pub fn run(&mut self) {
        let bot = &mut self.bot;
        loop {
            let allowed_moves = bot.allowed_tiles();
            let win_state = bot.status();
            let cur_color = bot.current_color();
            let is_self_move = cur_color == bot.self_color();

            if win_state.is_over() {
                break;
            }

            if allowed_moves.len() > 0 {
                if is_self_move {
                    let pl_move = bot.run_ai();
                    bot.apply_move(&pl_move);
                    Chan::send(CLIMove::Coord(pl_move.0));
                } else {
                    let pl_move = loop {
                        let coord = Chan::read().coord();
                        let pl_move =
                            allowed_moves.iter().find(|(ti, _)| *ti == coord);
                        if let Some(pl_move) = pl_move {
                            break pl_move;
                        }
                    };
                    bot.apply_move(&pl_move);
                }
            } else {
                if is_self_move {
                    Chan::send(CLIMove::Pass);
                } else {
                    Chan::read();
                }
            }
            bot.set_color(cur_color.opposite());
            // log!(self, "{:?}", self.board);
        }
    }

    fn report(&self) {
        // log!(
        //     self,
        //     "{}",
        //     match self.win_state {
        //         EndState::Tie => "Tie!",
        //         EndState::BlackWon => "Black won!",
        //         EndState::WhiteWon => "White won!",
        //         _ => "Game hadn't been completed.",
        //     }
        // );
        // if let Some(logfile) = &self.log_file {
        //     let lck = logfile.lock().unwrap();
        //     lck.borrow_mut().flush().unwrap();
        // }
    }
}
