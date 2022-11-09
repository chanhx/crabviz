use snafu::ResultExt;

mod analysis;
mod graph;
mod handler;

use {
    self::analysis::Analyzer,
    crate::{
        error::{self, Result},
        lang,
    },
    handler::{handle_not_found, serve_static, serve_svg, Context},
    std::{path::Path, process::Child},
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

pub(crate) fn run(lsp_server: Child, path: &Path) -> Result<()> {
    let lang = Box::new(lang::Rust {});
    let (analyzer, _io_thread) = Analyzer::new(lang, lsp_server, path);

    use_state!(Context {
        root: path.to_string_lossy().to_string(),
        analyzer
    });
    asset_dir!("./src/app/assets");

    run!("localhost:8090")
        .map_err(Into::into)
        .context(error::RuntimeSnafu)
}
