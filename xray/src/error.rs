use failure::Fail;
use serde_json::Error as JsonError;
use std::io::Error as IOError;

#[derive(Debug, Fail)]
pub enum Error {
    /// Returned for general IO errors
    #[fail(display = "IO Error")]
    IO(IOError),
    /// Returned for serialization related errors
    #[fail(display = "Json Error")]
    Json(JsonError),
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Self {
        Error::Json(err)
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error::IO(err)
    }
}
