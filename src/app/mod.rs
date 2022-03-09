mod handler;

use std::path::Path;

use anyhow::Result;
use vial::prelude::*;

use crate::analysis::Analyzer;
use handler::{handle_not_found, serve_static, serve_svg, Context};

routes! {
    GET "/" => serve_svg;
    GET "/assets/*path" => serve_static;
    GET "/*path" => handle_not_found;
}

pub(crate) fn run(path: &Path) -> Result<()> {
    let analyzer = Analyzer::new(path);

    use_state!(Context {
        root: path.to_string_lossy().to_string(),
        analyzer
    });
    asset_dir!("./src/app/assets");
    run!("localhost:8090")?;

    Ok(())
}
