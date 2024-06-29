use std::collections::HashMap;

use marshal::context::Context;
use marshal::de::Deserialize;
use marshal_core::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView};
use marshal_core::Primitive;

pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl<'de, P: Decoder> Deserialize<P> for JsonValue {
    fn deserialize<'p>(p: AnyDecoder<'p, P>, mut ctx: Context) -> anyhow::Result<Self> {
        match p.decode(DecodeHint::Any)? {
            DecoderView::Primitive(Primitive::Bool(x)) => Ok(JsonValue::Bool(x)),
            DecoderView::Primitive(Primitive::F64(x)) => Ok(JsonValue::Number(x)),
            DecoderView::Primitive(Primitive::Unit) => Ok(JsonValue::Null),
            DecoderView::String(x) => Ok(JsonValue::String(x.decode_cow()?.into_owned())),
            DecoderView::Seq(mut p) => {
                let mut vec = vec![];
                while let Some(next) = p.decode_next()? {
                    vec.push(<JsonValue as Deserialize<P>>::deserialize(
                        next,
                        ctx.reborrow(),
                    )?);
                }
                Ok(JsonValue::Array(vec))
            }
            DecoderView::Map(mut p) => {
                let mut map = HashMap::new();
                while let Some(mut entry) = p.decode_next()? {
                    let key = entry
                        .decode_key()?
                        .decode(DecodeHint::String)?
                        .try_into_string()?
                        .decode_cow()?
                        .into_owned();
                    let value = <JsonValue as Deserialize<P>>::deserialize(
                        entry.decode_value()?,
                        ctx.reborrow(),
                    )?;
                    entry.decode_end()?;
                    map.insert(key, value);
                }
                Ok(JsonValue::Object(map))
            }
            v => v.mismatch("json-like value")?,
        }
    }
}
