// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use chess::{
    definitions::NumberOf,
    pieces::{ALL_PIECES, PIECE_NAMES, Piece},
    square::Square,
};
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::ParallelProgressIterator;
use parameters::Parameters;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use textplots::{Chart, Plot, Shape};
use tuner::Tuner;
use tuner_score::TuningScore;
use tuning_position::TuningPosition;

use crate::epd_parser::WdlModel;
use crate::offsets::Offsets;
mod epd_parser;
mod interleave;
mod math;
mod offsets;
mod parameters;
mod tracing_values;
mod tuner;
mod tuner_score;
mod tuning_position;

#[derive(Parser, Debug)]
#[command(version, about="Texel tuner for HCE in byte-knight", long_about=None)]
struct Options {
    #[command(subcommand)]
    command: Command,
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ParameterStartType {
    Zero,
    EngineValues,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum WdlModelArg {
    /// Auto-detect: 0.0/0.5/1.0 = white-relative, else side-to-move.
    Auto,
    /// All results are from white's perspective.
    WhiteRelative,
    /// All results are from the side-to-move's perspective.
    SideToMove,
}

impl From<WdlModelArg> for WdlModel {
    fn from(arg: WdlModelArg) -> Self {
        match arg {
            WdlModelArg::Auto => WdlModel::Auto,
            WdlModelArg::WhiteRelative => WdlModel::WhiteRelative,
            WdlModelArg::SideToMove => WdlModel::SideToMove,
        }
    }
}

const INPUT_DATA_HELP: &str = "Filtered, marked EPD or 'book' input data.";
#[derive(Subcommand, Debug)]
enum Command {
    Tune {
        #[clap(short, long, help = INPUT_DATA_HELP)]
        input_data: String,
        #[clap(short, long, help = "Number of epochs to run.")]
        epochs: Option<usize>,
        #[arg(value_enum, short, long, help = "How to start the parameters", default_value_t = ParameterStartType::Zero)]
        param_start_type: ParameterStartType,
        #[arg(value_enum, short, long, help = "How to interpret WDL game results", default_value_t = WdlModelArg::Auto)]
        wdl_model: WdlModelArg,
    },
    PlotK {
        #[clap(short, long, help = INPUT_DATA_HELP)]
        input_data: String,
    },
    ComputeError {
        #[clap(short, long, help = INPUT_DATA_HELP)]
        input_data: String,
        #[clap(
            short,
            long,
            help = "k value to compute error for (0.009)",
            default_value_t = 0.009
        )]
        k: f64,
    },
    Bench {
        #[clap(short, long, help = "Number of epochs to run.", default_value_t = 50)]
        epochs: usize,
        #[clap(short, long, help = INPUT_DATA_HELP, default_value = "data/lichess-test.book")]
        input_data: String,
    },
    /// Interleave multiple datasets into a single EPD file for tuning.
    Interleave {
        #[clap(
            short,
            long,
            help = "Path to TOML config file specifying datasets to interleave."
        )]
        config: String,
    },
}

fn print_table(indent: usize, table: &[TuningScore]) {
    for rank in 0..8 {
        for file in 0..8 {
            let idx = rank * 8 + file;
            if file == 0 {
                print!("{:indent$}", "", indent = indent);
            }
            let val = table[idx];
            print!("{val:?}, ");
            if file == 7 {
                println!();
            }
        }
    }
}

fn print_params(params: &Parameters) {
    println!("Tuned parameters:");
    println!("=================");
    println!("pub const PSQTS : [[PhasedScore; Square::COUNT]; Piece::COUNT] = [");
    for piece in ALL_PIECES {
        println!("    // {}", PIECE_NAMES[piece as usize]);
        println!("    [");
        let start_idx = piece as usize * Square::COUNT;
        let end_index = start_idx + Square::COUNT;
        let table = &params.as_slice()[start_idx..end_index];
        print_table(8, table);
        println!("    ],");
    }
    println!("];");
    println!();

    // Print out the passed pawn bonus value
    println!("pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [",);

    for rank in 0..NumberOf::PASSED_PAWN_RANKS {
        let idx = Offsets::PASSED_PAWN + rank;
        let val = params.as_slice()[idx];
        println!("    {val:?}, ");
    }

    println!("];");

    println!();

    // Print out the doubled pawn penalty values
    println!("pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [");

    for file in 0..NumberOf::FILES {
        let idx = Offsets::DOUBLED_PAWN + file;
        let val = params.as_slice()[idx];
        println!("    {val:?}, ");
    }

    println!("];");

    println!();

    println!("pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [");

    for file in 0..NumberOf::FILES {
        let idx = Offsets::ISOLATED_PAWN + file;
        let val = params.as_slice()[idx];
        println!("    {val:?}, ");
    }

    println!("];");
    println!();
    println!(
        "pub const BISHOP_PAIR_BONUS: PhasedScore = {:?};",
        params.as_slice()[Offsets::BISHOP_PAIR]
    );

    println!();
    println!("pub const KING_SAFETY: [PhasedScore; Piece::COUNT - 1] =");
    print!("    [");
    for piece_idx in Piece::iter().filter(|&p| p != Piece::King) {
        let idx = Offsets::KING_SAFETY + piece_idx as usize - 1;
        let val = params.as_slice()[idx];
        print!("{val:?}, ");
    }
    println!("];");

    println!();
    println!("pub const PAWN_THREAT: [PhasedScore; Piece::COUNT] = [");
    for piece_idx in Piece::iter() {
        let idx = Offsets::PAWN_THREAT + piece_idx as usize;
        let val = params.as_slice()[idx];
        println!("    {val:?}, //{}", PIECE_NAMES[piece_idx as usize]);
    }
    println!("];");

    println!();
    println!("pub const KNIGHT_THREAT: [PhasedScore; Piece::COUNT] = [");
    for piece_idx in Piece::iter() {
        let idx = Offsets::KNIGHT_THREAT + piece_idx as usize;
        let val = params.as_slice()[idx];
        println!("    {val:?}, //{}", PIECE_NAMES[piece_idx as usize]);
    }
    println!("];");

    println!();
    println!("pub const BISHOP_THREAT: [PhasedScore; Piece::COUNT] = [");
    for piece_idx in Piece::iter() {
        let idx = Offsets::BISHOP_THREAT + piece_idx as usize;
        let val = params.as_slice()[idx];
        println!("    {val:?}, //{}", PIECE_NAMES[piece_idx as usize]);
    }
    println!("];");

    println!();
    println!("pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [");
    for mobility in 0..=NumberOf::KNIGHT_MOVES {
        let idx = Offsets::offset_for_mobility(Piece::Knight, mobility);
        let val = params.as_slice()[idx];
        println!("    {val:?},");
    }
    println!("];");

    println!();
    println!("pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [");
    for mobility in 0..=NumberOf::BISHOP_MOVES {
        let idx = Offsets::offset_for_mobility(Piece::Bishop, mobility);
        let val = params.as_slice()[idx];
        println!("    {val:?},");
    }
    println!("];");

    println!();
    println!("pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [");
    for mobility in 0..=NumberOf::ROOK_MOVES {
        let idx = Offsets::offset_for_mobility(Piece::Rook, mobility);
        let val = params.as_slice()[idx];
        println!("    {val:?},");
    }
    println!("];");

    println!();
    println!("pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [");
    for mobility in 0..=NumberOf::QUEEN_MOVES {
        let idx = Offsets::offset_for_mobility(Piece::Queen, mobility);
        let val = params.as_slice()[idx];
        println!("    {val:?},");
    }
    println!("];");

    println!();
    println!("// Small bonus for being the side to move.");
    println!(
        "pub const TEMPO_BONUS: PhasedScore = {:?};",
        params.as_slice()[Offsets::offset_for_tempo_bonus()]
    );

    println!();
    println!("pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [");
    for file in 0..NumberOf::FILES {
        let idx = Offsets::offset_for_rook_open_file(file as u8);
        let val = params.as_slice()[idx];
        println!("    {val:?},");
    }
    println!("];");

    println!();
    println!("pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [");
    for file in 0..NumberOf::FILES {
        let idx = Offsets::offset_for_rook_semi_open_file(file as u8);
        let val = params.as_slice()[idx];
        println!("    {val:?},");
    }
    println!("];");

    let pawn_shield_storm_row_comments = ["King file", "Left adjacent", "Right adjacent"];
    println!();
    println!(
        "pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = ["
    );
    for (file_idx, comment) in pawn_shield_storm_row_comments
        .iter()
        .enumerate()
        .take(NumberOf::KING_FLANK_FILES)
    {
        println!("    // {}", comment);
        print!("    [");
        for rank_idx in 0..NumberOf::PAWN_SHIELD_RANKS {
            let idx = Offsets::offset_for_pawn_shield(file_idx, rank_idx);
            let val = params.as_slice()[idx];
            print!("{val:?}, ");
        }
        println!("],");
    }
    println!("];");

    println!();
    println!(
        "pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = ["
    );
    for (file_idx, comment) in pawn_shield_storm_row_comments
        .iter()
        .enumerate()
        .take(NumberOf::KING_FLANK_FILES)
    {
        println!("    // {}", comment);
        print!("    [");
        for rank_idx in 0..NumberOf::PAWN_STORM_RANKS {
            let idx = Offsets::offset_for_pawn_storm(file_idx, rank_idx);
            let val = params.as_slice()[idx];
            print!("{val:?}, ");
        }
        println!("],")
    }
    println!("];");
}

fn plot_k(tuner: &Tuner) {
    let mut points = Vec::new();
    let data_point_count = 1_000;
    let k_min = 0.;
    let k_max = 0.1;
    (0..data_point_count)
        .into_par_iter()
        .progress_count(data_point_count as u64)
        .map(|val| {
            let k = val as f64 / data_point_count as f64 * (k_max - k_min) + k_min;
            let error = tuner.mean_square_error(k);
            (k as f32, error as f32)
        })
        .collect_into_vec(&mut points);

    Chart::new(180, 60, k_min as f32, k_max as f32)
        .lineplot(&Shape::Points(points.as_slice()))
        .nice();
}

fn parse_data(input_data: &str, wdl_model: WdlModel) -> Vec<TuningPosition> {
    println!("Reading data from: {input_data}");
    let positions = epd_parser::parse_epd_file(input_data, wdl_model);
    println!("Read {} positions", positions.len());
    positions
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(std::thread::available_parallelism().unwrap().get())
        .build_global()
        .unwrap();

    let options = Options::parse();
    match options.command {
        Command::Tune {
            input_data,
            epochs,
            param_start_type,
            wdl_model,
        } => {
            let positions = parse_data(&input_data, wdl_model.into());
            let parameters = match param_start_type {
                ParameterStartType::Zero => Parameters::default(),
                ParameterStartType::EngineValues => Parameters::create_from_engine_values(),
            };
            let epchs = epochs.unwrap_or(10_000);
            println!("Tuning parameters from {param_start_type:?} for {epchs} epochs",);
            let mut tuner = tuner::Tuner::new(parameters, &positions, epchs);
            let tuned_results = tuner.tune();
            print_params(tuned_results);
        }
        Command::PlotK { input_data } => {
            let positions = parse_data(&input_data, WdlModel::Auto);
            let parameters = Parameters::create_from_engine_values();
            let tuner = tuner::Tuner::new(parameters, &positions, 10_000);
            plot_k(&tuner);
        }
        Command::ComputeError { input_data, k } => {
            let positions = parse_data(&input_data, WdlModel::Auto);
            let parameters = Parameters::create_from_engine_values();
            let tuner = tuner::Tuner::new(parameters, &positions, 10_000);
            let error = tuner.mean_square_error(k);
            println!("Error for k {k:.8}: {error:.8}");
        }
        Command::Bench { epochs, input_data } => {
            let read_start = std::time::Instant::now();
            let positions = parse_data(&input_data, WdlModel::Auto);
            let read_elapsed = read_start.elapsed();
            println!(
                "Read {} in {:.3}s",
                positions.len(),
                read_elapsed.as_secs_f64()
            );

            let parameters = Parameters::create_from_engine_values();
            let mut tuner = tuner::Tuner::new(parameters, &positions, epochs);

            println!("Computing optimal K value...");
            let k = tuner.compute_k();
            println!("Optimal K value: {k:.8}");

            let mse_start = tuner.mean_square_error(k);
            println!("Running {epochs} epochs...");
            let start = std::time::Instant::now();
            for _ in 0..epochs {
                tuner.run_epoch(k);
            }
            let elapsed = start.elapsed();
            let mse_end = tuner.mean_square_error(k);
            println!("MSE Diff: {:.5}", mse_start - mse_end);
            println!(
                "Total: {:.3}s | Per epoch: {:.3}ms",
                elapsed.as_secs_f64(),
                elapsed.as_secs_f64() * 1000.0 / epochs as f64
            );
        }
        Command::Interleave { config } => {
            interleave::run_interleave(&config);
        }
    }
}
