use crate::{
    analysis::File,
    graph::{self, CallMap},
};

pub(super) fn serve_svg(files: &Vec<File>, calls: &CallMap) -> String {
    let contents = graph::gen_graph(files, calls).unwrap();

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" type="text/css" href="assets/styles.css">
    <script src="assets/svg-pan-zoom.min.js"></script>
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
