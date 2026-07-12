use anyhow::Result;
use clap::Parser;
use guided_review::{Cli, execute};

fn main() -> Result<()> {
    let output = execute(Cli::parse())?;
    println!("{}", output.display());
    Ok(())
}
