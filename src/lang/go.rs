use {
    super::{Entry, Language},
    std::{
        path::Path,
        process::{Child, Command, Stdio},
    },
};

pub(crate) struct Go {}

impl Language for Go {
    fn start_language_server(&self) -> Child {
        Command::new("gopls")
            .arg("serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start the language server")
    }

    fn entry(&self, base: &Path) -> Entry {
        Entry::new(base, vec!["go".to_string()], &[".git"])
    }
}
