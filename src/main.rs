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
        .expect("`--lang` argument is required at present")
        .to_owned();
    let lang = language_handler(&lang);

    app::run(lang, &path)
        .map_err(Into::into)
        .context(error::RuntimeSnafu)
}
