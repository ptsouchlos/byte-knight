// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    collections::{HashMap, hash_map},
    path::{Path, PathBuf},
    process::exit,
};

use chess::{board::Board, fen};
use console::Emoji;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

static CHECK_BOX: Emoji = Emoji("✅", "");
static CROSS_MARK: Emoji = Emoji("❌", "");

use std::error::Error;

#[derive(Debug, serde::Deserialize)]
pub struct LichessPuzzleRecord {
    #[serde(rename = "FEN")]
    pub(crate) fen: String,
}

pub fn read_lichess_puzzles(path_buf: PathBuf) -> Result<Vec<LichessPuzzleRecord>, Box<dyn Error>> {
    let reader = csv::Reader::from_path(path_buf);
    let records = reader?
        .deserialize()
        .collect::<Result<Vec<LichessPuzzleRecord>, _>>()?;

    Ok(records)
}

fn decompress_data(output_data_path: &Path, compressed_data_path: &Path) -> anyhow::Result<()> {
    let mut decompress_command = std::process::Command::new("zstd");
    decompress_command
        .arg("-d")
        .arg(compressed_data_path.to_str().unwrap())
        .arg("-o")
        .arg(output_data_path.to_str().unwrap());
    println!("Decompressing data file...");
    println!("Executing command: {decompress_command:?}");
    decompress_command.spawn()?.wait()?;

    // check if the output file exists
    if !output_data_path.exists() {
        return Err(anyhow::anyhow!(
            "Failed to decompress data file: output file not found"
        ));
    }

    Ok(())
}

fn main() {
    // load data
    let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    data_path.push("data/lichess_db_puzzle.csv");
    if !data_path.exists() {
        println!("Data file not found, decompressing from .zst file...");
        let mut zst_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        zst_path.push("data/lichess_db_puzzle.csv.zst");
        let decompress_result = decompress_data(&data_path, &zst_path);
        if decompress_result.is_err() {
            println!(
                "Failed to decompress data file: {:?}",
                decompress_result.err()
            );
            exit(-1);
        }
    }

    assert!(data_path.exists());
    println!("Reading test data...");
    let records_result = read_lichess_puzzles(data_path);

    // Compare two FEN strings for equality only using the first four parts
    let fen_match = |fen_left: &String, fen_right: &String| -> bool {
        let fen_left_result = fen::split_fen_string(fen_left);
        let fen_right_result = fen::split_fen_string(fen_right);
        if fen_left_result.is_err() || fen_right_result.is_err() {
            return false;
        }

        let fen_left_parts = fen_left_result.unwrap();
        let fen_right_parts = fen_right_result.unwrap();

        if fen_left_parts.len() != fen_right_parts.len() {
            return false;
        }

        for part in 0..4 {
            if fen_left_parts[part] != fen_right_parts[part] {
                return false;
            }
        }

        true
    };

    match records_result {
        Ok(records) => {
            let mut hashes: Vec<(u64, String)> = Vec::with_capacity(records.len());
            println!("Read {} records", records.len());
            println!("Calculating hashes...");
            records
                .par_iter()
                .progress_count(records.len() as u64)
                .map(|record| {
                    let board = Board::from_fen(&record.fen);
                    assert!(board.is_ok());
                    let board = board.unwrap();
                    let hash = board.zobrist_hash();
                    (hash, record.fen.clone())
                })
                .collect_into_vec(&mut hashes);

            // Compare the hashes
            println!("Comparing hashes...");
            let mut hash_map: HashMap<u64, Vec<String>> = std::collections::HashMap::new();

            for (hash, fen) in hashes {
                if let hash_map::Entry::Vacant(e) = hash_map.entry(hash) {
                    e.insert(vec![fen]);
                } else {
                    let vec = hash_map.get_mut(&hash).unwrap();
                    vec.push(fen);
                }
            }

            let mut duplicates = 0;
            for (hash, fens) in hash_map {
                if fens.len() > 1 {
                    let mut matched = false;
                    for i in 0..fens.len() {
                        for j in i + 1..fens.len() {
                            if fen_match(&fens[i], &fens[j]) {
                                matched = true;
                                break;
                            }
                        }
                        if matched {
                            break;
                        }
                    }

                    if !matched {
                        println!("Hash collision detected: {hash}");
                        for fen in fens {
                            println!("{fen}");
                        }
                        duplicates += 1;
                    }
                }
            }

            if duplicates == 0 {
                println!("{CHECK_BOX} No hash collisions detected!");
            } else {
                println!("{CROSS_MARK} {duplicates} hash collisions detected");
            }
        }
        Err(e) => {
            println!("Failed to read records: {e:?}");
            exit(-1);
        }
    }
}
