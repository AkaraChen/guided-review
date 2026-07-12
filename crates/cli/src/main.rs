use anyhow::Result;
use clap::Parser;
use guided_review::{Cli, execute};

fn main() -> Result<()> {
    execute(Cli::parse())
}
