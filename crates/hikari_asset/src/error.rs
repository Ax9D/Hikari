use std::ffi::OsString;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Extension {0:?} not supported")]
    UnsupportedModelFormat(OsString),
    #[error("Unsupported image format {0:?} on file : {1:?}")]
    UnsupportedImageFormat(String, String),
    #[error("Couldn't detect file format for file: {0:?}")]
    FailedToIdentifyFormat(OsString),
    #[error("Failed to parse file {0:?}, Error:")]
    FailedToParse(OsString, String),
}
