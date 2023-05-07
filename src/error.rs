use {
    snafu::{prelude::*, Backtrace},
    std::io::Error as IoError,
};

pub type Result<T> = std::result::Result<T, Error>;
type AnotherError = Box<dyn std::error::Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
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
}
