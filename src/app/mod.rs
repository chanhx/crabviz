use snafu::ResultExt;

mod analysis;
mod graph;
mod handler;

use {
    self::analysis::Analyzer,
    crate::{
        error::{self, Result},
        lang::Language,
    },
    handler::{handle_not_found, serve_static, serve_svg, Context},
    std::path::Path,
    vial::prelude::*,
};

pub(crate) use {
    analysis::FileOutline,
    graph::{Edge, GenerateSVG, Node, Subgraph, TableNode},
};

routes! {
    GET "/" => serve_svg;
    GET "/assets/*path" => serve_static;
    GET "/*path" => handle_not_found;
}

pub(crate) fn run(lang: Box<dyn Language + Sync + Send>, path: &Path) -> Result<()> {
    let lsp_server = lang.start_language_server();
    let (analyzer, _io_thread) = Analyzer::new(lang, lsp_server, path);

    use_state!(Context {
        root: path.to_string_lossy().to_string(),
        analyzer
    });
    asset_dir!("./src/app/assets");

    run!("localhost:8090")
        .map_err(Into::into)
        .context(error::AppServerSnafu)
}
