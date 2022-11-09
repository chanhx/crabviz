mod client;
mod message;

use {
    shellexpand,
    std::process::{Child, Command, Stdio},
};

pub(crate) use {client::Client, message::Message};

pub(crate) fn start_language_server(server_path: &str) -> Child {
    let server_path = shellexpand::full(server_path)
        .map(|path| std::path::Path::new(path.as_ref()).canonicalize().unwrap())
        .unwrap();

    Command::new(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start the language server")
}
