use clap::Parser;
use std::path::PathBuf;
use anyhow::{Context, Result};
use regex::Regex;

/// Search for a pattern in a file and display the lines that contain it.


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The pattern to look for
    pattern: String,
    /// The path to the file to read
    path: PathBuf,
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {

    // Parse command line arguments
    let args = Cli::parse();

    // Read the file content
    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("Could not read file `{}`", args.path.display()))?;

    // Search for the pattern (case-insensitive) and display matching lines with line numbers
    let max_width = content.lines().count().to_string().len(); // Calculate width for line numbers
    let re = Regex::new(&format!(r"(?i){}", regex::escape(&args.pattern)))?;
    for (num, line) in content.lines().enumerate() {
        if re.is_match(line) {
            println!("[{:>width$}] {line}", num + 1, width = max_width);
        }
    }

    if args.verbose {
        println!("pattern: {:?}, path: {:?}", args.pattern, args.path);
    }

    Ok(())
}
