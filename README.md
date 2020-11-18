# Reversi (Othello) engine

## There are currently 2 implementations of AI in this repo:
- [MiniMax](https://en.wikipedia.org/wiki/Minimax)
- [MCTS](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search) ([single-level tree](src/mcts.rs) and [multi-level](src/mcts2.rs))


You can select one with `--bot-impl` command-line parameter

## There are two game modes: Regular reversi and Anti-reversi
The default mode is anti-reversi and can be turned to regular with
`--no-anti` parameter


#### For more usage options, see `--help`
