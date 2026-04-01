use clap::{Args, Parser, Subcommand, arg};

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
}

#[derive(Args, Debug)]
pub struct TestArgs {
    #[arg(long, short, default_value = None)]
    /// Test solver with an easy to solve cipher data so solution can be built quickly. Mutually exclusive together with hard.
    pub easy: Option<bool>,
    /// Test solver with hard to solve cipher data, solution will take time to be built. Mutually exclusive together with easy.
    #[arg(long, default_value_t = false)]
    pub hard: bool,
}
