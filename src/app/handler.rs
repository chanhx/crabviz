use vial::prelude::*;

use crate::{
    analysis::File,
    graph::{self, CallMap},
};

pub(super) struct Context {
    pub files: Vec<File>,
    pub refs: CallMap,
}

pub(super) fn serve_svg(req: Request) -> impl Responder {
    let ctx = req.state::<Context>();

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
        graph::gen_graph(&ctx.files, &ctx.refs).unwrap(),
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
