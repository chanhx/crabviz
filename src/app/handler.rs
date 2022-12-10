use {
    crate::{
        analysis::{Analyzer, Relations, SymbolLocation},
        graph::{dot::Dot, GenerateSVG},
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

    let files = analyzer.file_outlines(std::path::Path::new(&path));

    let paths = files
        .iter()
        .map(|outline| &outline.path)
        .collect::<HashSet<_>>();

    let calls = analyzer
        .outgoing_calls(&files)
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
                    .map(|call| SymbolLocation::new(&call.to.uri, &call.to.selection_range.start))
                    .collect(),
            )
        })
        .collect::<Relations>();

    let implementations = analyzer
        .interface_implementations(&files)
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
                    .map(|location| location)
                    .collect(),
            )
        })
        .collect::<Relations>();

    let tables = files
        .iter()
        .map(|f| analyzer.lang.file_repr(f))
        .collect::<Vec<_>>();

    let calling_edges = analyzer
        .lang
        .calling_repr(calls, &analyzer.path_map.borrow());
    let impl_edges = analyzer
        .lang
        .impl_repr(implementations, &analyzer.path_map.borrow());
    let edges = [calling_edges, impl_edges].concat();

    let subgraphs = analyzer.subgraphs(&files);

    let dot = Dot {};
    let svg = dot.generate_svg(&tables, &edges, &subgraphs);

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
