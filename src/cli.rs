use std::{error::Error, path::PathBuf};

use clap::{Args, Parser, Subcommand, arg};

use crate::solver::enigma_settings::EnigmaSettings;

/// A program to break Enigma cipher
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Print debug information through the solving process
    #[arg(long, short)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Tests enigma solver using example cipher and plain data
    Test(TestArgs),
    /// Solve enigma given cipher text and partial plain text by finding configuration used for encryption.
    Solve(SolveArgs),
    /// Encrypt/Decrypt
    #[command(alias = "decrypt")]
    Encrypt(EncryptArgs),
}

fn parse_settings(s: &str) -> Result<EnigmaSettings, Box<dyn Error + Send + Sync>> {
    s.parse()
}

#[derive(Args, Debug)]
pub struct TestArgs {
    #[arg(long, short, default_value_t = false)]
    /// Test solver with an easy to solve cipher data so solution can be built quickly. Mutually exclusive together with hard.
    pub easy: bool,
    #[arg(long, default_value_t = false)]
    /// Test solver with hard to solve cipher data, solution will take time to be built. Mutually exclusive together with easy.
    pub hard: bool,
    #[arg(short, long, default_value_t = false)]
    /// When set, run multithreaded solver (set by default)
    pub single_threaded: bool,
}

#[derive(Args, Debug)]
pub struct SolveArgs {
    /// Path to file containing cipher text
    pub cipher_text: PathBuf,
    /// Path to file containing plain text associated with cipher text
    pub plain_text: PathBuf,
    /// Save results to a RON file
    #[arg(short = 'r', long)]
    pub result_settings_file: Option<PathBuf>,
    /// Run single threaded
    #[arg(short, long, default_value_t = false)]
    pub single_threaded: bool,
}

#[derive(Args, Debug)]
pub struct EncryptArgs {
    /// RON/JSON settings file to set an enigma before encrypting
    #[arg(short = 's', long)]
    pub settings_file: Option<PathBuf>,
    /// Ron/JSON settings from argument
    #[arg(long, value_parser=parse_settings)]
    pub settings: Option<EnigmaSettings>,
}
