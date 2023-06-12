use std::fmt::Display;

use serde::{de, ser};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("unexpected end of input")]
    Eof,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("string length of {0} does not fit into a u16")]
    StrLen(usize),
    #[error("sequence length of {0} does not fit into a i32")]
    SeqLen(usize),
    #[error("{0}")]
    Mutf8(#[from] mutf8::error::Error),
    #[error("did not detect a valid ShadeNBT header")]
    InvalidHeader,
    #[error("field name is unset")]
    FieldInfoUnset,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
