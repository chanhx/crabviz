use {
    confy::ConfyError,
    snafu::{prelude::*, Backtrace},
    std::io::Error as IoError,
};

pub type Result<T> = std::result::Result<T, Error>;
type AnotherError = Box<dyn std::error::Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("The path to {} language server is not set", lang))]
    ServerPathNotSet { backtrace: Backtrace, lang: String },

    #[snafu(display("Error reading config: {}", source))]
    ReadConfig {
        backtrace: Backtrace,
        source: ConfyError,
    },

    #[snafu(display("Project path is not valid: {}", source))]
    PathNotValid {
        backtrace: Backtrace,
        source: IoError,
    },

    #[snafu(display("Error generating svg: {}", source))]
    GenerateSVG {
        backtrace: Backtrace,
        source: AnotherError,
    },

    #[snafu(display("Error running server: {}", source))]
    AppServer {
        backtrace: Backtrace,
        source: AnotherError,
    },
}
