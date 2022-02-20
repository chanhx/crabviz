mod handler;

use std::path::Path;

use anyhow::Result;
use vial::prelude::*;

use crate::{
    analysis::Analyzer,
    file_structure::{File, RItemType},
    graph::CallMap,
};

routes! {
    GET "/" => |req| {
        let context = req.state::<Context>();
        handler::serve_svg(&context.files, &context.calls)
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
    calls: CallMap,
}

pub(crate) fn run(path: &Path) -> Result<()> {
    let analyzer = Analyzer::new(&path);
    let files = analyzer.analyze(&path)?;

    let funcs = files
        .iter()
        .flat_map(|m| m.items.iter())
        .filter(|ritem| matches!(ritem.ty, RItemType::Func))
        .collect::<Vec<_>>();

    let methods = files
        .iter()
        .flat_map(|m| m.items.iter())
        .filter(|ritem| matches!(ritem.ty, RItemType::Impl))
        .flat_map(|ritem| {
            let children = ritem.children.as_ref().unwrap();
            children.iter().collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let calls = [funcs, methods]
        .concat()
        .iter()
        .map(|f| (f.pos.clone(), analyzer.find_callers(&f.pos).unwrap()))
        .collect::<CallMap>();

    use_state!(Context { files, calls });

    asset_dir!("./src/app/assets");
    run!("localhost:8090")?;

    Ok(())
}
