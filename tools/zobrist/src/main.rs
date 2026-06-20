// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html
use clap::Parser;

use std::process::exit;

mod generate;
mod verify;

#[derive(Debug, clap::Subcommand)]
enum Command {
    Verify(verify::VerifyArgs),
    Generate(generate::GenerateArgs),
}

#[derive(Parser)]
struct Options {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    let options = Options::parse();
    match options.command {
        Command::Verify(args) => {
            if let Err(e) = verify::execute(args) {
                println!("Error executing command: {e}");
                exit(-1);
            } else {
                println!("Verification successful!");
            }
        }
        Command::Generate(args) => {
            if let Err(e) = generate::execute(args) {
                println!("Error executing command: {e}");
                exit(-1);
            } else {
                println!("Generation successful!");
            }
        }
    }
}
