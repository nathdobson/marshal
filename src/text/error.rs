use std::char::{CharTryFromError, DecodeUtf16Error};
use std::num::{ParseFloatError, ParseIntError};
use std::str::Utf8Error;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum TextError {
    Eof,
    ExpectedToken { expected: char, found: Option<char> },
    UnexpectedInput,
    Poisoned,
    Utf8Error,
    ParseIntError,
    ParseFloatError,
    BadNumber,
    CharTryFromError,
    FromUtf8Error,
    StringContainsControl,
    StringBadEscape,
    UnexpectedIdentifer,
    UnexpectedInitialCharacter { found: char},
    BadState,
    ExpectedString,
    TrailingText,
    DecodeUtf16Error,
    DepthBudgetExceeded,
}

impl From<Utf8Error> for TextError {
    fn from(_: Utf8Error) -> Self {
        TextError::Utf8Error
    }
}

impl From<ParseIntError> for TextError {
    fn from(_: ParseIntError) -> Self {
        TextError::ParseIntError
    }
}

impl From<ParseFloatError> for TextError {
    fn from(_: ParseFloatError) -> Self {
        TextError::ParseFloatError
    }
}

impl From<CharTryFromError> for TextError {
    fn from(_: CharTryFromError) -> Self {
        TextError::CharTryFromError
    }
}

impl From<FromUtf8Error> for TextError {
    fn from(_: FromUtf8Error) -> Self {
        TextError::FromUtf8Error
    }
}

impl From<DecodeUtf16Error> for TextError {
    fn from(_: DecodeUtf16Error) -> Self {
        TextError::DecodeUtf16Error
    }
}

pub type TextResult<T> = Result<T, TextError>;
