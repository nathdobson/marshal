use crate::text::any::{TextAny, TextAnyParser, TextAnyPosition};
use crate::text::depth_budget::DepthBudget;
use crate::text::error::{TextError, TextResult};
use crate::text::TextParser;
use serde_json::{Number, Value};

pub fn into_json_value(p: &mut TextParser) -> TextResult<Value> {
    let result = into_json_value_rec(TextAnyParser::new(
        p,
        TextAnyPosition::Any,
        DepthBudget::new(100),
    ))?;
    p.end_parsing()?;
    Ok(result)
}
pub fn into_json_value_rec(p: TextAnyParser) -> TextResult<Value> {
    match p.parse_any()? {
        TextAny::Number(p) => Ok(Value::Number(
            Number::from_f64(p.parse_number()?).ok_or(TextError::BadNumber)?,
        )),
        TextAny::TextSeqParser(mut p) => {
            let mut vec = vec![];
            while let Some(next) = p.next()? {
                vec.push(into_json_value_rec(next)?);
            }
            p.end()?;
            Ok(Value::Array(vec))
        }
        TextAny::String(x) => Ok(Value::String(x)),
        TextAny::Null => Ok(Value::Null),
        TextAny::TextMapParser(mut p) => {
            let mut map = serde_json::Map::new();
            while let Some(mut entry) = p.next()? {
                let key = match entry.parse_key()?.parse_any()? {
                    TextAny::String(key) => key,
                    _ => unreachable!(),
                };
                let value = into_json_value_rec(entry.parse_value()?)?;
                map.insert(key, value);
            }
            p.end()?;
            Ok(Value::Object(map))
        }
        TextAny::Bool(x) => Ok(Value::Bool(x)),
    }
}
