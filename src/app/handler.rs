use crate::{
    file_structure::Module,
    graph::{self, CallMap},
};

pub(super) fn serve_svg(modules: &Vec<Module>, calls: &CallMap) -> String {
    let contents = graph::gen_graph(modules, calls).unwrap();

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
