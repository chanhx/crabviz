mod dot;

use anyhow::Result;

use crate::file_structure::Module;

pub(crate) use dot::CallMap;

pub(crate) fn gen_graph(m: &Vec<Module>, calls: &CallMap) -> Result<String> {
    let dot_graph = dot::modules_graph(m, calls);
    dot::render_svg(dot_graph)
}
