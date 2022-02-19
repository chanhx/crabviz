mod analysis;
mod app;
mod file_structure;
mod graph;

use std::env;

use anyhow::Result;

fn main() -> Result<()> {
    let path = env::current_dir()?;

    app::run(&path)
}
