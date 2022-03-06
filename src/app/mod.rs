mod handler;

use std::{collections::HashMap, path::Path};

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

    let files_id = analyzer.files_id(path);
    let files = files_id
        .iter()
        .map(|&id| analyzer.file_structure(id))
        .collect::<Vec<_>>();
    let refs = files
        .iter()
        .flat_map(|file| analyzer.file_references(file))
        .collect::<HashMap<_, _>>();

    use_state!(Context { files, refs });
    asset_dir!("./src/app/assets");
    run!("localhost:8090")?;

    Ok(())
}
