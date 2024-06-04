use crate::context::DeserializeContext;
use crate::deserialize::Deserialize;
use crate::error::ParseError;
use crate::{AnyParser, EntryParser, MapParser, ParseHint, Parser, ParserView, SeqParser};
use serde_json::{Number, Value};

impl<'de, P: Parser<'de>> Deserialize<'de, P> for Value {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &DeserializeContext) -> Result<Self, ParseError> {
        match p.parse(ParseHint::Any)? {
            ParserView::Bool(x) => Ok(Value::Bool(x)),
            ParserView::F64(x) => Ok(Value::Number(Number::from_f64(x).unwrap())),
            ParserView::String(x) => Ok(Value::String(x)),
            ParserView::Unit => Ok(Value::Null),
            ParserView::Seq(mut p) => {
                let mut vec = vec![];
                while let Some(next) = p.parse_next()? {
                    vec.push(<Value as Deserialize<'de, P>>::deserialize(next, ctx)?);
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
                    let value = <Value as Deserialize<'de, P>>::deserialize(entry.parse_value()?, ctx)?;
                    map.insert(key, value);
                }
                Ok(Value::Object(map))
            }
            _ => todo!(),
        }
    }
}
