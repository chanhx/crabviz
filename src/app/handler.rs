// use crate::graph::modules_graph;
use crate::{file_structure::Module, graph};

pub(super) fn serve_svg(modules: &Vec<Module>) -> String {
    let contents = graph::gen_graph(modules).unwrap();

    format!(
        r#"
<!DOCTYPE html>
<html>
<body>
    {}
</body>
</html>
        "#,
        contents,
    )
}
