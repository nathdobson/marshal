use serde_json::{Number, Value};

use crate::depth_budget::{DepthBudgetParser, OverflowError, WithDepthBudget};
use crate::error::ParseError;
use crate::poison::{PoisonAnyParser, PoisonError, PoisonParser, PoisonState};
use crate::simple::{SimpleAnyParser, SimpleParserAdapter};
use crate::json::error::JsonError;
use crate::json::{SingletonContext, JsonParser};
use crate::{AnyParser, EntryParser, MapParser, ParseHint, Parser, ParserView, SeqParser};

pub fn into_json_value<'p, 'de>(p: &'p mut JsonParser<'de>) -> Result<Value, ParseError> {
    let mut poison = PoisonState::new();
    let result = into_json_value_rec::<
        PoisonParser<DepthBudgetParser<SimpleParserAdapter<JsonParser<'de>>>>,
    >(PoisonAnyParser::new(
        &mut poison,
        WithDepthBudget::new(100, SimpleAnyParser::new(p, SingletonContext::default())),
    ))?;
    poison.check()?;
    p.end_parsing()?;
    Ok(result)
}
pub fn into_json_value_rec<'p, 'de, P: Parser<'de>>(
    p: P::AnyParser<'p>,
) -> Result<Value, ParseError> {
    match p.parse(ParseHint::Any)? {
        // TextAny::Number(p) => Ok(Value::Number(
        //     Number::from_f64(p.parse_number()?).ok_or(TextError::BadNumber)?,
        // )),
        // TextAny::TextSeqParser(mut p) => {
        //     let mut vec = vec![];
        //     while let Some(next) = p.next()? {
        //         vec.push(into_json_value_rec(next)?);
        //     }
        //     p.end()?;
        //     Ok(Value::Array(vec))
        // }
        // TextAny::String(x) => Ok(Value::String(x)),
        // TextAny::Null => Ok(Value::Null),
        // TextAny::TextMapParser(mut p) => {
        // }
        // TextAny::Bool(x) => Ok(Value::Bool(x)),
        ParserView::Bool(x) => Ok(Value::Bool(x)),
        ParserView::F64(x) => Ok(Value::Number(Number::from_f64(x).unwrap())),
        ParserView::String(x) => Ok(Value::String(x)),
        ParserView::Unit => Ok(Value::Null),
        ParserView::Seq(mut p) => {
            let mut vec = vec![];
            while let Some(next) = p.parse_next()? {
                vec.push(into_json_value_rec::<P>(next)?);
            }
            Ok(Value::Array(vec))
        }
        ParserView::Map(mut p) => {
            let mut map = serde_json::Map::new();
            while let Some(mut entry) = p.parse_next()? {
                let key = match entry.parse_key()?.parse(ParseHint::String)? {
                    ParserView::String(key) => key,
                    _ => unreachable!(),
                };
                let value = into_json_value_rec::<P>(entry.parse_value()?)?;
                map.insert(key, value);
            }
            Ok(Value::Object(map))
        }
        _ => todo!(),
    }
}
