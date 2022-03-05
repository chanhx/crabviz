mod handler;

use std::{collections::HashMap, path::Path, time::Instant};

use anyhow::Result;
use vial::prelude::*;

use crate::{
    analysis::{Analyzer, File},
    graph::CallMap,
};

routes! {
    GET "/" => |req| {
        let context = req.state::<Context>();
        handler::serve_svg(&context.files, &context.refs)
    };
    GET "/assets/*path" => |req| {
        let path = req.arg("path").unwrap_or("");
        Response::from_asset(path)
    };

    GET "/*path" => |req|
        Response::from(404).with_body(format!(
            "<h1>404 Not Found: {}</h1>",
            req.arg("path").unwrap_or("")
        ));
}

struct Context {
    files: Vec<File>,
    refs: CallMap,
}

pub(crate) fn run(path: &Path) -> Result<()> {
    let now = Instant::now();

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

    let elapsed = now.elapsed();
    println!("elapsed {} seconds", elapsed.as_secs());

    use_state!(Context { files, refs });
    asset_dir!("./src/app/assets");
    run!("localhost:8090")?;

    Ok(())
}
