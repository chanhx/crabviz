use std::collections::HashMap;

use vial::prelude::*;

use crate::{analysis::Analyzer, graph};

pub(super) struct Context {
    pub root: String,
    pub analyzer: Analyzer,
}

unsafe impl Sync for Context {}

pub(super) fn serve_svg(req: Request) -> impl Responder {
    let ctx = req.state::<Context>();
    let path = ctx.root.clone();
    let analyzer = &ctx.analyzer;

    let files_id = analyzer.files_id(path);
    let files = files_id
        .iter()
        .map(|&id| analyzer.file_structure(id))
        .collect::<Vec<_>>();
    let refs = files
        .iter()
        .flat_map(|file| analyzer.file_references(file))
        .collect::<HashMap<_, _>>();

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" type="text/css" href="assets/styles.css">
    <script src="assets/path-data-polyfill.min.js"></script>
    <script src="assets/svg-pan-zoom.min.js"></script>
</head>
<body>
    {}
    <script src="assets/preprocess.js"></script>
</body>
</html>
        "#,
        graph::gen_graph(&files, &refs).unwrap(),
    )
}

pub(super) fn serve_static(req: Request) -> impl Responder {
    let path = req.arg("path").unwrap_or("");
    Response::from_asset(path)
}

pub(super) fn handle_not_found(req: Request) -> impl Responder {
    Response::from(404).with_body(format!(
        "<h1>404 Not Found: {}</h1>",
        req.arg("path").unwrap_or("")
    ))
}
