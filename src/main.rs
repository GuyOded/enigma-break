mod solver;
use clap::Parser;
use colog::default_builder;
use log::LevelFilter;

use crate::cli::Cli;
mod cli;
mod commands;

fn main() {
    let mut builder = default_builder();
    let cli = Cli::parse();
    if cli.debug {
        builder.filter(None, LevelFilter::Debug);
    };

    builder.init();

    match cli.command {
        cli::Commands::Test(test_args) => commands::run_test(test_args),
        cli::Commands::Solve {
            cipher_text,
            plain_text,
            settings_file,
        } => todo!(),
        cli::Commands::Encrypt {
            settings_file,
            settings,
        } => todo!(),
    }
}
