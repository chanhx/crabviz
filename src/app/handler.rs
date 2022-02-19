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
<head>
    <link rel="stylesheet" type="text/css" href="assets/styles.css">
</head>
<body>
    {}
    <script src="assets/preprocess.js"></script>
</body>
</html>
        "#,
        contents,
    )
}