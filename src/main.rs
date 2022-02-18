mod analysis;
mod file_structure;

use anyhow::Result;
use std::env;

use analysis::Analyzer;

fn main() -> Result<()> {
    let path = env::current_dir()?;
    let analyzer = Analyzer::new(&path);
    let modules = analyzer.analyze(&path)?;

    println!("{}", modules.len());

    Ok(())
}
