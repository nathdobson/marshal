use serde_json::{Number, Value};

use crate::de::context::DeserializeContext;
use crate::de::Deserialize;
use crate::parse::{AnyParser, EntryParser, MapParser, ParseHint, Parser, ParserView, SeqParser};
use crate::Primitive;

impl<'de, P: Parser<'de>> Deserialize<'de, P> for Value {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &DeserializeContext) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Any)? {
            ParserView::Primitive(Primitive::Bool(x)) => Ok(Value::Bool(x)),
            ParserView::Primitive(Primitive::F64(x)) => {
                Ok(Value::Number(Number::from_f64(x).unwrap()))
            }
            ParserView::Primitive(Primitive::Unit) => Ok(Value::Null),
            ParserView::String(x) => Ok(Value::String(x)),
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
                    let value =
                        <Value as Deserialize<'de, P>>::deserialize(entry.parse_value()?, ctx)?;
                    map.insert(key, value);
                }
                Ok(Value::Object(map))
            }
            x => todo!("{:?}", x),
        }
    }
}