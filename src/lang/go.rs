use super::Language;

pub(crate) struct Go;

impl Language for Go {
    fn should_filter_out_file(&self, file: &str) -> bool {
        file.ends_with("_test.go")
    }
}
