use crate::{
    file_structure::File,
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
