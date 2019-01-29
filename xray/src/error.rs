use failure::Fail;
use serde_json::Error as JsonError;
use std::io::Error as IOError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO Error")]
    IO(IOError),
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
