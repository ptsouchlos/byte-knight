use std::{
    collections::{HashMap, hash_map},
    path::{Path, PathBuf},
};

use anyhow::bail;
use chess::{board::Board, fen};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

#[derive(clap::Args, Debug)]
pub(crate) struct VerifyArgs {
    /// The path to the file to verify.
    #[arg(short, long)]
    file: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct LichessPuzzleRecord {
    #[serde(rename = "FEN")]
    pub(crate) fen: String,
}

/// Helper to read the Lichess puzzle data from a CSV file.
/// The CSV file is expected to have a header row with a "FEN" column.
fn read_lichess_puzzles(path_buf: PathBuf) -> anyhow::Result<Vec<LichessPuzzleRecord>> {
    let reader = csv::Reader::from_path(path_buf);
    let records = reader?
        .deserialize()
        .collect::<Result<Vec<LichessPuzzleRecord>, _>>()?;

    Ok(records)
}

/// Helper to decompress a zstd compressed file using the `zstd` command line tool.
/// The output file will be created at `output_data_path`.
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

pub(crate) fn execute(args: VerifyArgs) -> anyhow::Result<()> {
    // Load the data if it exists.
    let mut data_path = PathBuf::from(args.file);
    if !data_path.exists() {
        bail!("Data file not found: {:?}", data_path);
    }

    // Is it compressed? If so, decompress it first.
    if let Some(ext) = data_path.extension()
        && ext == "zst"
    {
        println!("Data file compressed, decompressing...");
        let zst_path = data_path.clone();
        let output_path = data_path.with_extension("csv");
        let decompress_result = decompress_data(&output_path, &zst_path);
        if decompress_result.is_err() {
            bail!(
                "Failed to decompress data file: {:?}",
                decompress_result.err()
            )
        }

        data_path = output_path;
    }

    // Check that the data file exists after decompression.
    assert!(data_path.exists());
    println!("Reading test data...");
    // Read the records from the data file.
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
                Ok(())
            } else {
                bail!("{duplicates} hash collisions detected");
            }
        }
        Err(e) => {
            bail!("Failed to read records: {e:?}")
        }
    }
}
