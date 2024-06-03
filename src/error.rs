use crate::Parser;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub type ParseError = anyhow::Error;
pub type ParseResult<T> = Result<T, ParseError>;
