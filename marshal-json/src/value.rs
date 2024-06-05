use marshal::de::context::DeserializeContext;
use marshal::de::Deserialize;
use marshal::parse::{AnyParser, EntryParser, MapParser, ParseHint, Parser, ParserView, SeqParser};
use marshal::Primitive;
use std::collections::HashMap;

pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl<'de, P: Parser<'de>> Deserialize<'de, P> for JsonValue {
    fn deserialize<'p>(p: P::AnyParser<'p>, ctx: &DeserializeContext) -> anyhow::Result<Self> {
        match p.parse(ParseHint::Any)? {
            ParserView::Primitive(Primitive::Bool(x)) => Ok(JsonValue::Bool(x)),
            ParserView::Primitive(Primitive::F64(x)) => Ok(JsonValue::Number(x)),
            ParserView::Primitive(Primitive::Unit) => Ok(JsonValue::Null),
            ParserView::String(x) => Ok(JsonValue::String(x)),
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
                    let key = match entry.parse_key()?.parse(ParseHint::String)? {
                        ParserView::String(key) => key,
                        _ => unreachable!(),
                    };
                    let value =
                        <JsonValue as Deserialize<'de, P>>::deserialize(entry.parse_value()?, ctx)?;
                    map.insert(key, value);
                }
                Ok(JsonValue::Object(map))
            }
            x => todo!("{:?}", x),
        }
    }
}
