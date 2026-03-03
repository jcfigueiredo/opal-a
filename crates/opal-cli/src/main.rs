use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

/// The Opal programming language
#[derive(Debug, Parser)]
#[command(name = "opal")]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run an Opal source file
    Run {
        /// Path to the .opl source file
        file: PathBuf,
    },

    /// Start an interactive REPL
    Repl,

    /// Run the spec test suite
    Test {
        /// Optional path to test file or directory
        path: Option<PathBuf>,
    },

    /// Run benchmarks
    Bench {
        /// Optional path to benchmark file or directory
        path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            eprintln!("opal run: not yet implemented (file: {})", file.display());
            process::exit(1);
        }
        Commands::Repl => {
            eprintln!("opal repl: not yet implemented");
            process::exit(1);
        }
        Commands::Test { path } => {
            eprintln!(
                "opal test: not yet implemented (path: {:?})",
                path.as_deref().map(|p| p.display())
            );
            process::exit(1);
        }
        Commands::Bench { path } => {
            eprintln!(
                "opal bench: not yet implemented (path: {:?})",
                path.as_deref().map(|p| p.display())
            );
            process::exit(1);
        }
    }
}
