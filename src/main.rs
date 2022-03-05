mod analysis;
mod app;
mod graph;

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// path to rust project
    #[clap(parse(try_from_str))]
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = fs::canonicalize(&args.path)?;

    app::run(&path)
}
