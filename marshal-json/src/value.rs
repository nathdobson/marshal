use std::collections::HashMap;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal_core::decode::{AnyParser, EntryParser, MapParser, ParseHint, Parser, ParserView, SeqParser};
use marshal_core::Primitive;

pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl<'de, P: Parser<'de>> Deserialize<'de, P> for JsonValue {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &mut Context) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Any)? {
            ParserView::Primitive(Primitive::Bool(x)) => Ok(JsonValue::Bool(x)),
            ParserView::Primitive(Primitive::F64(x)) => Ok(JsonValue::Number(x)),
            ParserView::Primitive(Primitive::Unit) => Ok(JsonValue::Null),
            ParserView::String(x) => Ok(JsonValue::String(x.into_owned())),
            ParserView::Seq(mut p) => {
                let mut vec = vec![];
                while let Some(next) = p.parse_next()? {
                    vec.push(<JsonValue as Deserialize<'de, P>>::deserialize(next, ctx)?);
                }
                Ok(JsonValue::Array(vec))
            }
            ParserView::Map(mut p) => {
                let mut map = HashMap::new();
                while let Some(mut entry) = p.parse_next()? {
                    let key = entry
                        .parse_key()?
                        .parse(ParseHint::String)?
                        .try_into_string()?
                        .into_owned();
                    let value =
                        <JsonValue as Deserialize<'de, P>>::deserialize(entry.parse_value()?, ctx)?;
                    entry.parse_end()?;
                    map.insert(key, value);
                }
                Ok(JsonValue::Object(map))
            }
            x => todo!("{:?}", x),
        }
    }
}
