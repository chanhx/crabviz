use {
    clap::{arg, Command},
    std::path::Path,
};

pub fn cli(config_path: &Path) -> Command {
    let pkg_name = env!("CARGO_PKG_NAME");

    Command::new(pkg_name)
        .bin_name(pkg_name)
        .arg_required_else_help(true)
        .arg(arg!(path: <PATH> "A directory to be analyzed"))
        .arg(arg!(-l --lang <LANGUAGE> "Main programming language of the project"))
        .arg(arg!(-s --server <PATH> "Path to the language server"))
        .after_help(format!(
            "Configuration:\r\n{}",
            config_path.to_str().unwrap()
        ))
}
