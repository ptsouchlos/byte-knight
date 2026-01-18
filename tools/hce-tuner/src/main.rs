// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use chess::{
    definitions::NumberOf,
    pieces::{ALL_PIECES, PIECE_NAMES},
};
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::ParallelProgressIterator;
use parameters::Parameters;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use textplots::{Chart, Plot, Shape};
use tuner::Tuner;
use tuner_score::TuningScore;
use tuning_position::TuningPosition;

use crate::offsets::Offsets;
mod epd_parser;
mod math;
mod offsets;
mod parameters;
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
    PieceValues,
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
    println!("pub const PSQTS : [[PhasedScore; NumberOf::SQUARES]; NumberOf::PIECE_TYPES] = [");
    for piece in ALL_PIECES {
        println!("    // {}", PIECE_NAMES[piece as usize]);
        println!("    [");
        let start_idx = piece as usize * NumberOf::SQUARES;
        let end_index = start_idx + NumberOf::SQUARES;
        let table = &params.as_slice()[start_idx..end_index];
        print_table(8, table);
        println!("    ],");
    }
    println!("];");
    println!();

    // Print out the passed pawn bonus value
    println!("pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [",);

    for rank in 0..NumberOf::PASSED_PAWN_RANKS {
        let idx = Offsets::PASSED_PAWN as usize + rank;
        let val = params.as_slice()[idx];
        println!("    {val:?}, ");
    }

    println!("];");

    println!();

    // Print out the doubled pawn penalty values
    println!("pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [");

    for file in 0..NumberOf::FILES {
        let idx = Offsets::DOUBLED_PAWN as usize + file;
        let val = params.as_slice()[idx];
        println!("    {val:?}, ");
    }

    println!("];");

    println!();

    println!("pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [");

    for file in 0..NumberOf::FILES {
        let idx = Offsets::ISOLATED_PAWN as usize + file;
        let val = params.as_slice()[idx];
        println!("    {val:?}, ");
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

fn parse_data(input_data: &str) -> Vec<TuningPosition> {
    println!("Reading data from: {input_data}");
    let positions = epd_parser::parse_epd_file(input_data);
    // let positions = get_positions();
    println!("Read {} positions", positions.len());
    positions
}

fn main() {
    let options = Options::parse();
    match options.command {
        Command::Tune {
            input_data,
            epochs,
            param_start_type,
        } => {
            let positions = parse_data(&input_data);
            let parameters = match param_start_type {
                ParameterStartType::Zero => Parameters::default(),
                ParameterStartType::EngineValues => Parameters::create_from_engine_values(),
                ParameterStartType::PieceValues => Parameters::create_from_piece_values(),
            };
            let epchs = epochs.unwrap_or(10_000);
            println!("Tuning parameters from {param_start_type:?} for {epchs} epochs",);
            let mut tuner = tuner::Tuner::new(parameters, &positions, epchs);
            let tuned_results = tuner.tune();
            print_params(tuned_results);
        }
        Command::PlotK { input_data } => {
            let positions = parse_data(&input_data);
            let parameters = Parameters::create_from_engine_values();
            let tuner = tuner::Tuner::new(parameters, &positions, 10_000);
            plot_k(&tuner);
        }
        Command::ComputeError { input_data, k } => {
            let positions = parse_data(&input_data);
            let parameters = Parameters::create_from_engine_values();
            let tuner = tuner::Tuner::new(parameters, &positions, 10_000);
            let error = tuner.mean_square_error(k);
            println!("Error for k {k:.8}: {error:.8}");
        }
    }
}
