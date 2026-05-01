<center><h1> byte-knight </h1></center>

[![codecov](https://codecov.io/gh/ptsouchlos/byte-knight/graph/badge.svg?token=USEPKU8K4G)](https://codecov.io/gh/ptsouchlos/byte-knight)

`byte-knight` is a UCI compliant chess engine written in Rust. It started as a port of the chess engine I submitted for Sebatian Lague's [Chess Engine Challenge](https://github.com/ptsouchlos/Leonidas) where it placed in the top 32 out of 600+ entries.

You can challenge `byte-knight` yourself on [Lichess](https://lichess.org/@/byte-knight)!

## Overview

`byte-knight` is my first "real" Rust project. I'm a long time [C++ developer](https://github.com/ptsouchlos?tab=repositories&q=&type=source&language=c%2B%2B&sort=stargazers) and have been itching to learn Rust. I really enjoyed participating in the chess challenge a while back and thought that writing a new chess engine from scratch would be a good way to learn the language.

`byte-knight` is a command line chess engine and does not come with any sort of user interface. There are many [chess GUIs](https://www.chessprogramming.org/GUI) out there that you can use like [cutechess](https://github.com/cutechess/cutechess).

New features are tested on an [OpenBench](https://github.com/AndyGrant/OpenBench) intance using [SPRT](https://github.com/jw1912/SPRT/blob/main/SPRT.md#how-sprt-actually-works) testing.

## Strength

| Version                                                            | Estimate | [CCRL 40/15](https://computerchess.org.uk/ccrl/4040/) | [CCRL Blitz](https://computerchess.org.uk/ccrl/404/) |
| ------------------------------------------------------------------ | -------- | ----------------------------------------------------- | ---------------------------------------------------- |
| [3.0.0](https://github.com/ptsouchlos/byte-knight/releases/tag/v3.0.0) | -        | 2386                                                  | 2311                                                 |
| 4.0.0                                                              | 2800     |                                                       |                                                      |

## Features

### Board/Game Representation

- Bitboard board representation
- "Magic" bitboards or PEXT for sliding piece attacks
- Zobrist hashing with board state history
- Legal and pseudo-legal move generator with support for staged move generation

### Search

- [Iterative deepening](https://www.chessprogramming.org/Iterative_Deepening)
- [Negamax](https://www.chessprogramming.org/Negamax) with alpha/beta pruning
- [Quiescence search](https://www.chessprogramming.org/Quiescence_Search)
- [Transposition Table](https://www.chessprogramming.org/Transposition_Table)
- [Principle variation search](https://www.chessprogramming.org/Principal_Variation_Search)
- [Aspiration windows](https://www.chessprogramming.org/Aspiration_Windows)
- [Reverse futility pruning](https://www.chessprogramming.org/Reverse_Futility_Pruning)
- [Late Move Reductions](https://www.chessprogramming.org/Late_Move_Reductions)
- [Internal Iterative Reductions](https://www.chessprogramming.org/Internal_Iterative_Reductions)
- [Null Move Pruning](https://www.chessprogramming.org/Null_Move_Pruning)
- [Late Move Pruning](https://cosmo.tardis.ac/files/2023-02-20-viri-wiki.html#futility-pruning-late-move-pruning)
- [Futility Pruning](https://cosmo.tardis.ac/files/2023-02-20-viri-wiki.html#futility-pruning-late-move-pruning)
- [Check Extensions](https://www.chessprogramming.org/Check_Extensions)
- [Razoring](https://www.chessprogramming.org/Razoring)
- [QS Delta Pruning](https://www.chessprogramming.org/Delta_Pruning)
- [Time control](https://www.chessprogramming.org/Time_Management)
  - Basic hard limit scaling based on remaining time.
  - Soft limits scaling based on best move stability.
- Move ordering via a [move picker](https://www.chessprogramming.org/Move_Generation)
  - [TT Moves](https://www.chessprogramming.org/Transposition_Table#Priority_by_Move_Ordering_Position)
  - [MVV/LVA](https://www.chessprogramming.org/MVV-LVA) with transposition table priority
  - [History heuristic](https://www.chessprogramming.org/History_Heuristic) with history gravity
  - [Killer move heuristic](https://www.chessprogramming.org/Killer_Heuristic)
  - [Static exchange evaluation](https://www.chessprogramming.org/Static_Exchange_Evaluation) for good/bad tacticals.
  - Staged move generation (TT, good tacticals, quiet moves, bad tacticals)

### Evaluation

- Piece square tables with tapered evaluation using [PeSTO](https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function) values
- Pawn structure (doubled, isolated and passed pawns)
- Bishop pair bonus
- King safety
- Pawn storm and shield
- Piece mobility
- Rook open/semi-open files
- Tempo bonus
- Threat evaluation

Project includes a HCE tuner based on [jw1912/hce-tuner](https://github.com/jw1912/hce-tuner) and modified for use in `byte-knight`. HCE values have been trained on the `lichess-big3-resolved` dataset interleaved with data from [Clockwork](https://data.cwchess.org/).

### UCI

[UCI](https://www.chessprogramming.org/UCI) is a standard protocol for chess engines. `byte-knight` implements the following commands:

- `uci`
- `ucinewgame`
- `isready`
- `position <fen> moves <move list>`
- `go`
  - `depth <depth>`
  - `nodes <nodes>`
  - `wtime <wtime> btime <btime> winc <winc> binc <binc>`
  - `movetime <movetime>`
- `setoption name <name> value <value>` - Configure a UCI option (see [UCI Options](#uci-options)).
- `stop`
- `quit`
- `debug <on|off>` - Turn debug mode on or off. In debug mode, more information is printed during search.
- `hash` - See TT stats and usage.
- `history` - See the contents of the history table.

### Other Commands

To see all commands that `byte-knight` supports, type:

```bash
byte-knight help
```

To see all options for a given command, type `byte-knight <cmd> --help`.

- `bench` - This runs a fixed depth search on a variety of positions. This is used by [OpenBench](https://github.com/AndyGrant/OpenBench) for scaling based on engine performance.
- `perft` - Run `perft` on a given FEN or EPD file for the given depth.
- `split-perft` - Run split perft for a given FEN.

## UCI Options

| Name    | Value Range | Default | Description                       |
| ------- | ----------- | ------- | --------------------------------- |
| Hash    | [1 - 1024]  | 16      | Set the TT table size in MB       |
| Threads | [1]         | 1       | How many threads to use in search |

## Build and Run

Clone the repo and run:

```bash
cargo -r run -p byte-knight
```

### Building on Apple Silicon (aarch64)

The TT prefetch optimization on Apple Silicon requires the nightly toolchain. To enable it for a local checkout, run the following command in the project root:

```bash
rustup override set nightly
```

### Development Dependencies

To run the full suite of supported tests, benchmarks and other development dependencies, you will need the following tools (in addition to Rust and Cargo):

- [just](https://github.com/casey/just)
- Rust llvm-profdata component
  - Install with `rustup component add llvm-tools-preview`
- [grcov](https://github.com/mozilla/grcov) (Used to generate code coverage reports)
- [lcov](https://github.com/linux-test-project/lcov) (Required for `genhtml` to create HTML reports from `lcov` data)

## License

The project is licensed under the GPL license. See [LICENSE](LICENSE) for more details.

## Credits

Thanks/acknowledgement for those who have inspired and helped with this project:

- Sebastian Lague for his chess YouTube videos and for hosting a fun coding challenge.
- The [Chess Programming Wiki](https://www.chessprogramming.org/Main_Page) for all the free information. Thank you to all the various authors.
- Analog-Hors for some excellent write ups on chess, especially regarding magic numbers.
- Many members of the Engine Programming discord for helping see how little I really know.
- [Danny Hammer](https://github.com/dannyhammer/toad) for providing feedback, for helping me with troubleshooting my engine and for writing the `chessie` and `uci-parser` crates. Thanks for inspiring some of the techniques and methods used in `byte-knight`.
- [Marcel Vanthoor](https://github.com/mvanthoor/rustic) for his Rustic engine and associated [book](https://rustic-chess.org).
- Everyone at [pyrobench](https://pyronomy.pythonanywhere.com) for donating CPU time as well as helping me when I get stuck.
- Everyone at MattBench for donating CPU time as well as helping me when I get stuck.

## Author

| [<img src="https://avatars0.githubusercontent.com/u/6591180?s=460&v=4" width="100"><br><sub>@ptsouchlos</sub>](https://github.com/ptsouchlos) |
| :-------------------------------------------------------------------------------------------------------------------------------------------: |
