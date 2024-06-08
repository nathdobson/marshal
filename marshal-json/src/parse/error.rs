use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use marshal_core::decode::DecodeHint;

use crate::parse::any::PeekType;

#[derive(Debug)]
pub enum JsonError {
    Eof,
    ExpectedToken { expected: char, found: Option<char> },
    UnexpectedInput,
    Utf8Error,
    ParseIntError,
    ParseFloatError,
    BadNumber,
    CharTryFromError,
    FromUtf8Error,
    StringContainsControl,
    StringBadEscape,
    UnexpectedIdentifer { found: Vec<u8> },
    UnexpectedInitialCharacter { found: char },
    BadState,
    ExpectedString,
    TrailingText,
    DecodeUtf16Error,
    DepthBudgetExceeded,
    TooManyChars,
    SchemaMismatch { hint: DecodeHint, found: PeekType },
    UnexpectedNull,
    BadOption,
}

impl Display for JsonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for JsonError {}
