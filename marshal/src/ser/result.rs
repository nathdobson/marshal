use marshal_core::encode::{AnyEncoder, Encoder};
use crate::context::Context;
use crate::ser::Serialize;

impl<E: Encoder, A: Serialize<E>, B: Serialize<E>> Serialize<E> for Result<A, B> {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        let name = "Result";
        let variants = &["Ok", "Err"];
        match self {
            Ok(a) => {
                let mut e = e.encode_tuple_variant(name, variants, 0, 1)?;
                a.serialize(e.encode_field()?, ctx)?;
                e.end()?;
                Ok(())
            }
            Err(b) => {
                let mut e = e.encode_tuple_variant(name, variants, 1, 1)?;
                b.serialize(e.encode_field()?, ctx)?;
                e.end()?;
                Ok(())
            }
        }
    }
}