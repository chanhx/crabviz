use {
    std::{ffi::OsStr, path::Path},
    walkdir::WalkDir,
};

pub(crate) fn infer_language(path: &Path) -> Option<String> {
    WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok().filter(|e| e.file_type().is_file()))
        .filter_map(|e| {
            let file_name = e.file_name();
            let extension = e.path().extension().and_then(OsStr::to_str);

            if may_be_rust(file_name, extension) {
                return Some("rust".to_string());
            }

            if may_be_golang(file_name, extension) {
                return Some("go".to_string());
            }

            None
        })
        .next()
}

fn may_be_rust(file_name: &OsStr, extension: Option<&str>) -> bool {
    file_name == "Cargo.toml" || extension == Some("rs")
}

fn may_be_golang(file_name: &OsStr, extension: Option<&str>) -> bool {
    file_name == "go.sum" || extension == Some("go")
}
