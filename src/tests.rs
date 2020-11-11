// #[cfg(test)]
// mod bot_tests {
//     use crate::{bot::Bot, game::*, utils::*};
//     use std::convert::TryFrom;

//     #[test]
//     fn check_winstate_1() {
//         let board = "BB_BBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB";
//         let game_state = GameState::try_from(board.to_string()).unwrap();
//         let mut bot = Bot {
//             game_state,
//             current_color: Cell::Black,
//             win_state: EndState::Unknown,
//             my_color: Cell::Black,
//             log_file: None,
//             max_tree_depth: 8,
//             is_anti: true,
//         };
//         bot.wincheck();
//         assert_eq!(bot.win_state, EndState::WhiteWon);
//     }

//     #[test]
//     fn check_winstate_2() {
//         let board = "BB_BBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBWBBBBB\
//                      BBBBBBBB";
//         let game_state = GameState::try_from(board.to_string()).unwrap();
//         let mut bot = Bot {
//             game_state,
//             current_color: Cell::Black,
//             win_state: EndState::Unknown,
//             my_color: Cell::Black,
//             log_file: None,
//             max_tree_depth: 8,
//             is_anti: true,
//         };
//         bot.wincheck();
//         assert_eq!(bot.win_state, EndState::OnePass);
//     }

//     #[test]
//     fn check_winstate_3() {
//         let board = "BB_BBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBBBBBBB\
//                      BBWBBBBB\
//                      BBBBBBBB";
//         let game_state = GameState::from_board(board.to_string()).unwrap();
//         let mut bot = Bot {
//             game_state,
//             current_color: Cell::Black,
//             win_state: EndState::Unknown,
//             my_color: Cell::Black,
//             log_file: None,
//             max_tree_depth: 8,
//             is_anti: true,
//         };
//         bot.wincheck();
//         assert_eq!(bot.win_state, EndState::OnePass);
//     }
// }
