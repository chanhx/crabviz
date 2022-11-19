use {
    crate::config::Config,
    clap::{arg, Command},
};

pub fn cli() -> Command {
    let pkg_name = env!("CARGO_PKG_NAME");

    let cmd = Command::new(pkg_name)
        .bin_name(pkg_name)
        .arg_required_else_help(true)
        .arg(arg!(path: <PATH> "A directory to be analyzed"))
        .arg(arg!(-l --lang <LANGUAGE> "Main programming language of the project"));

    match Config::path() {
        Ok(path) => cmd.after_help(format!("Configuration:\r\n{}", path.to_str().unwrap())),
        Err(_) => cmd,
    }
}
