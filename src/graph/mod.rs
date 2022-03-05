mod dot;

use anyhow::Result;

use crate::analysis::File;

pub(crate) use dot::CallMap;

pub(crate) fn gen_graph(m: &Vec<File>, calls: &CallMap) -> Result<String> {
    let dot_graph = dot::files_graph(m, calls);
    dot::render_svg(dot_graph)
}
