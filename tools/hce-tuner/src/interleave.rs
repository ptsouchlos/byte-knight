// Part of the byte-knight project.
// Interleaves multiple EPD datasets into a single output file for tuning.

use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use serde::Deserialize;

use crate::epd_parser::{self, WdlModel};

#[derive(Debug, Deserialize)]
struct InterleaveConfig {
    output: String,
    datasets: Vec<DatasetConfig>,
}

#[derive(Debug, Deserialize)]
struct DatasetConfig {
    path: String,
    wdl_model: WdlModelConfig,
    max_positions: usize,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum WdlModelConfig {
    WhiteRelative,
    SideToMove,
}

impl From<WdlModelConfig> for WdlModel {
    fn from(config: WdlModelConfig) -> Self {
        match config {
            WdlModelConfig::WhiteRelative => WdlModel::WhiteRelative,
            WdlModelConfig::SideToMove => WdlModel::SideToMove,
        }
    }
}

/// A parsed EPD position: FEN string and white-relative game result.
struct ParsedPosition {
    fen: String,
    result: f64,
}

/// Load positions from a single dataset, converting results to white-relative.
fn load_dataset(config: &DatasetConfig, config_dir: &Path) -> Vec<ParsedPosition> {
    let file_path = config_dir.join(&config.path);
    let display_path = file_path.display();
    let file = File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open dataset file: {display_path}"));
    let reader = BufReader::new(file);
    let wdl_model: WdlModel = config.wdl_model.into();

    let mut positions = Vec::new();
    for line in reader.lines() {
        if positions.len() >= config.max_positions {
            break;
        }
        let line = line.expect("Failed to read line");
        match epd_parser::process_epd_line(&line) {
            Ok((board, game_result)) => {
                let result = epd_parser::to_white_relative(&board, game_result, wdl_model);
                positions.push(ParsedPosition {
                    fen: board.to_fen(),
                    result,
                });
            }
            Err(e) => {
                println!("Warning skipping line: {line}: {e}");
            }
        }
    }

    positions
}

/// Round-robin interleave multiple position lists.
/// When a dataset is exhausted, it is skipped; remaining datasets continue.
fn round_robin_interleave(datasets: Vec<Vec<ParsedPosition>>) -> Vec<ParsedPosition> {
    let total: usize = datasets.iter().map(|d| d.len()).sum();
    let mut result = Vec::with_capacity(total);
    let mut iterators: Vec<std::vec::IntoIter<ParsedPosition>> =
        datasets.into_iter().map(|d| d.into_iter()).collect();

    loop {
        let mut any_produced = false;
        for iter in &mut iterators {
            if let Some(pos) = iter.next() {
                result.push(pos);
                any_produced = true;
            }
        }
        if !any_produced {
            break;
        }
    }

    result
}

pub(crate) fn run_interleave(config_path: &str) {
    let config_path = Path::new(config_path);
    let config_content = std::fs::read_to_string(config_path)
        .unwrap_or_else(|_| panic!("Failed to read config file: {}", config_path.display()));
    let config: InterleaveConfig = toml::from_str(&config_content)
        .unwrap_or_else(|e| panic!("Failed to parse config file: {e}"));

    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));

    println!("Interleaving {} dataset(s)...", config.datasets.len());

    let mut all_datasets = Vec::with_capacity(config.datasets.len());
    for (i, dataset_config) in config.datasets.iter().enumerate() {
        let positions = load_dataset(dataset_config, config_dir);
        println!(
            "  [{}] {} — loaded {} positions (max: {}, wdl: {:?})",
            i + 1,
            dataset_config.path,
            positions.len(),
            dataset_config.max_positions,
            dataset_config.wdl_model,
        );
        all_datasets.push(positions);
    }

    let interleaved = round_robin_interleave(all_datasets);
    println!("Total interleaved positions: {}", interleaved.len());

    let output_path = config_dir.join(&config.output);
    let mut out_file = File::create(&output_path)
        .unwrap_or_else(|_| panic!("Failed to create output file: {}", output_path.display()));

    for pos in &interleaved {
        writeln!(out_file, "{} [{}]", pos.fen, pos.result).expect("Failed to write to output file");
    }

    println!("Wrote output to: {}", output_path.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_equal_sizes() {
        let a = vec![
            ParsedPosition {
                fen: "a1".into(),
                result: 1.0,
            },
            ParsedPosition {
                fen: "a2".into(),
                result: 0.5,
            },
        ];
        let b = vec![
            ParsedPosition {
                fen: "b1".into(),
                result: 0.0,
            },
            ParsedPosition {
                fen: "b2".into(),
                result: 1.0,
            },
        ];
        let result = round_robin_interleave(vec![a, b]);
        let fens: Vec<&str> = result.iter().map(|p| p.fen.as_str()).collect();
        assert_eq!(fens, vec!["a1", "b1", "a2", "b2"]);
    }

    #[test]
    fn round_robin_unequal_sizes() {
        let a = vec![
            ParsedPosition {
                fen: "a1".into(),
                result: 1.0,
            },
            ParsedPosition {
                fen: "a2".into(),
                result: 0.5,
            },
            ParsedPosition {
                fen: "a3".into(),
                result: 0.0,
            },
        ];
        let b = vec![ParsedPosition {
            fen: "b1".into(),
            result: 0.0,
        }];
        let result = round_robin_interleave(vec![a, b]);
        let fens: Vec<&str> = result.iter().map(|p| p.fen.as_str()).collect();
        assert_eq!(fens, vec!["a1", "b1", "a2", "a3"]);
    }

    #[test]
    fn round_robin_empty_dataset() {
        let a = vec![ParsedPosition {
            fen: "a1".into(),
            result: 1.0,
        }];
        let b: Vec<ParsedPosition> = vec![];
        let result = round_robin_interleave(vec![a, b]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].fen, "a1");
    }
}
