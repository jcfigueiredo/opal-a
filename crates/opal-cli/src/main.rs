use std::path::{Path, PathBuf};
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
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading {}: {}", file.display(), e);
                    process::exit(1);
                }
            };

            let program = match opal_parser::parse(&source) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!(
                        "{}:{}: {}",
                        file.display(),
                        format_error_location(&e, &source),
                        e,
                    );
                    process::exit(1);
                }
            };

            let base_dir = file.parent().unwrap_or(Path::new("."));
            let mut interpreter = opal_interp::Interpreter::with_base_dir(base_dir);
            if let Err(e) = interpreter.run(&program) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        Commands::Repl => {
            use std::io::{self, BufRead, Write};

            let mut interpreter = opal_interp::Interpreter::new();
            let stdin = io::stdin();
            let mut buffer = String::new();
            let mut continuation = false;

            println!("Opal REPL v0.1.0 (type 'exit' to quit)");

            loop {
                if continuation {
                    print!("... ");
                } else {
                    print!("opal> ");
                }
                io::stdout().flush().unwrap();

                let mut line = String::new();
                if stdin.lock().read_line(&mut line).unwrap() == 0 {
                    break; // EOF
                }
                let line = line.trim_end_matches('\n').trim_end_matches('\r');

                if !continuation && line == "exit" {
                    break;
                }

                buffer.push_str(line);
                buffer.push('\n');

                // Check if we need more input (unclosed blocks)
                if needs_continuation(&buffer) {
                    continuation = true;
                    continue;
                }

                // Try to parse and eval
                match opal_parser::parse(&buffer) {
                    Ok(program) => match interpreter.run(&program) {
                        Ok(()) => {}
                        Err(e) => eprintln!("{}", e),
                    },
                    Err(e) => eprintln!("ParseError: {}", e),
                }

                buffer.clear();
                continuation = false;
            }
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

fn format_error_location(err: &opal_parser::ParseError, source: &str) -> String {
    match err {
        opal_parser::ParseError::UnexpectedToken { span, .. }
        | opal_parser::ParseError::InvalidFString { span, .. }
        | opal_parser::ParseError::LexError { span, .. } => {
            let (line, col) = opal_lexer::source_location(source, span.start);
            format!("{}:{}", line, col)
        }
        opal_parser::ParseError::UnexpectedEof { .. } => "end of file".to_string(),
    }
}

fn needs_continuation(input: &str) -> bool {
    let openers = [
        "def ", "if ", "class ", "module ", "do\n", "do ", "for ", "while ", "actor ", "macro ",
        "try\n", "match ",
    ];
    let mut depth: i32 = 0;
    for line in input.lines() {
        let trimmed = line.trim();
        for opener in &openers {
            if trimmed.starts_with(opener) || trimmed == opener.trim() {
                depth += 1;
            }
        }
        if trimmed == "end" {
            depth -= 1;
        }
    }
    depth > 0
}
