mod handler;

use std::path::Path;

use anyhow::Result;
use vial::prelude::*;

use crate::{analysis::Analyzer, file_structure::Module};

routes! {
    GET "/" => |req| {
        let context = req.state::<Context>();
        handler::serve_svg(&context.modules)
    };

    GET "/*path" => |req|
        Response::from(404).with_body(format!(
            "<h1>404 Not Found: {}</h1>",
            req.arg("path").unwrap_or("")
        ));
}

struct Context {
    modules: Vec<Module>,
}

pub(crate) fn run(path: &Path) -> Result<()> {
    let analyzer = Analyzer::new(&path);
    let modules = analyzer.analyze(&path)?;

    use_state!(Context { modules });

    run!("localhost:8090")?;

    Ok(())
}
