mod dot;

use anyhow::Result;

use crate::file_structure::Module;

pub(crate) fn gen_graph(m: &Vec<Module>) -> Result<String> {
    let dot_graph = dot::modules_graph(&m);
    dot::render_svg(dot_graph)
}
