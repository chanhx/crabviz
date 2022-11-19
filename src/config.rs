use snafu::ResultExt;

use {
    crate::error::{self, Result},
    lazy_static::lazy_static,
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, path::PathBuf},
};

lazy_static! {
    pub static ref CONFIG: Config = Config::get();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub servers: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            servers: [("rust".into(), "path/to/lsp_server".into())]
                .into_iter()
                .collect(),
        }
    }
}

impl Config {
    fn get() -> Self {
        confy::load::<Config>(env!("CARGO_PKG_NAME"), "config")
            .expect("failed to read configuration")
    }

    pub(crate) fn path() -> Result<PathBuf> {
        confy::get_configuration_file_path(env!("CARGO_PKG_NAME"), "config")
            .map_err(Into::into)
            .context(error::RuntimeSnafu)
    }
}
