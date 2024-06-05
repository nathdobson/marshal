use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use marshal::parse::ParseHint;
use crate::read::any::PeekType;

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
    UnexpectedIdentifer,
    UnexpectedInitialCharacter { found: char },
    BadState,
    ExpectedString,
    TrailingText,
    DecodeUtf16Error,
    DepthBudgetExceeded,
    TooManyChars,
    SchemaMismatch { hint: ParseHint, found: PeekType },
    UnexpectedNull,
}

impl Display for JsonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for JsonError {}