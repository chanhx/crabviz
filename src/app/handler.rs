use {
    crate::{
        analysis::{Analyzer, Relations, SymbolLocation},
        dot::Dot,
        graph::gen_svg,
    },
    std::{collections::HashSet, path::PathBuf},
    vial::prelude::*,
};

pub(super) struct Context {
    pub root: String,
    pub analyzer: Analyzer,
}

unsafe impl Sync for Context {}

pub(super) fn serve_svg(req: Request) -> impl Responder {
    let ctx = req.state::<Context>();
    let path = ctx.root.clone();
    let analyzer = &ctx.analyzer;

    let file_outlines = analyzer.file_outlines(std::path::Path::new(&path));

    let paths = file_outlines
        .iter()
        .map(|outline| &outline.path)
        .collect::<HashSet<_>>();

    let mut calls = analyzer
        .outgoing_calls(&file_outlines)
        .into_iter()
        .map(|(caller, callee)| {
            (
                caller,
                callee
                    .into_iter()
                    .filter(|callee| {
                        let path = callee.to.uri.path();
                        let path = PathBuf::from(path);

                        paths.contains(&path)
                    })
                    .map(|call| {
                        (
                            SymbolLocation::new(&call.to.uri, &call.to.selection_range.start),
                            None,
                        )
                    })
                    .collect(),
            )
        })
        .collect::<Relations>();

    let implementations = analyzer
        .interface_implementations(&file_outlines)
        .into_iter()
        .map(|(interface, implementations)| {
            (
                interface,
                implementations
                    .into_iter()
                    .filter(|location| {
                        let path = &location.path;
                        let path = PathBuf::from(path);

                        paths.contains(&path)
                    })
                    .map(|location| (location, None))
                    .collect(),
            )
        });

    calls.extend(implementations);

    let svg = gen_svg(&Dot {}, &file_outlines, calls);

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" type="text/css" href="assets/styles.css">
    <script src="assets/svg-pan-zoom.min.js"></script>
</head>
<body>
    {svg}
    <script src="assets/preprocess.js"></script>
</body>
</html>
        "#,
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
