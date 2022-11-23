mod app;
mod cli;
mod config;
mod dot;
mod error;
mod lang;
mod lsp;
mod utils;

use {cli::cli, error::Result, lang::language_handler, snafu::ResultExt, std::fs};

fn main() -> Result<()> {
    let matches = cli().get_matches();

    let path = fs::canonicalize(
        matches
            .get_one::<String>("path")
            .expect("`path` is required"),
    )
    .context(error::PathNotValidSnafu)?;

    let lang = matches
        .get_one::<String>("lang")
        .map(ToOwned::to_owned)
        .or(lang::infer_language(&path))
        .expect("can not infer the programming language, please provide a `--lang` argument");
    let lang = language_handler(&lang);

    app::run(lang, &path)
}
