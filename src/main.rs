mod app;
mod cli;
mod config;
mod dot;
mod error;
mod lang;
mod lsp;
mod utils;

use {
    cli::cli,
    config::Config,
    error::{Error, Result},
    snafu::ResultExt,
    std::fs,
};

fn main() -> Result<()> {
    let app_name = env!("CARGO_PKG_NAME");
    let config_path =
        confy::get_configuration_file_path(app_name, "config").context(error::ReadConfigSnafu)?;

    let config = confy::load::<Config>(app_name, "config").context(error::ReadConfigSnafu)?;

    let matches = cli(&config_path).get_matches();

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

    let server_path = matches
        .get_one::<String>("server")
        .or(config.servers.get(&lang))
        .ok_or(Error::ServerPathNotSet { lang })?;

    let lsp_server = lsp::start_language_server(server_path);

    app::run(lsp_server, &path)
        .map_err(Into::into)
        .context(error::RuntimeSnafu)
}
