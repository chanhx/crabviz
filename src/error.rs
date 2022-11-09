use {confy::ConfyError, snafu::prelude::*, std::io::Error as IoError};

pub type Result<T> = std::result::Result<T, Error>;
type AnotherError = Box<dyn std::error::Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("The path to {} language server is not set", lang))]
    ServerPathNotSet {
        lang: String,
    },

    #[snafu(display("Error reading config: {}", source))]
    ReadConfig {
        source: ConfyError,
    },

    #[snafu(display("Project path is not valid: {}", source))]
    PathNotValid {
        source: IoError,
    },

    Runtime {
        source: AnotherError,
    },
}
