mod solver;
use std::error::Error;

use clap::Parser;
use colog::default_builder;
use log::LevelFilter;

use crate::cli::Cli;
mod cli;
mod commands;

fn main() -> Result<(), Box<dyn Error>> {
    let mut builder = default_builder();
    let cli = Cli::parse();
    if cli.debug {
        builder.filter(None, LevelFilter::Debug);
    };

    builder.init();

    match cli.command {
        cli::Commands::Test(test_args) => commands::run_test(test_args),
        cli::Commands::Solve(args) => commands::solve(args),
        cli::Commands::Encrypt(args) => todo!(),
    }
}
