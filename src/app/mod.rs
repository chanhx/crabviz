mod graph;
mod handler;

use {
    crate::analysis::Analyzer,
    anyhow::Result,
    handler::{handle_not_found, serve_static, serve_svg, Context},
    std::path::Path,
    vial::prelude::*,
};

pub(crate) use graph::{GenerateSVG, Node, References, Subgraph, TableNode};

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
