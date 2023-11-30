use crate::cli::Cli;
use clap::Parser;

mod cli;

fn main() {
    let args = Cli::parse();

    println!("path: {:?}", args.path)
}
