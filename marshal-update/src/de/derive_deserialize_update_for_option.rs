use marshal::context::Context;
use marshal::de::Deserialize;
use marshal::decode::{AnyDecoder, Decoder};

use crate::de::DeserializeUpdate;

macro_rules! derive_deserialize_update_for_option {
    ($($ty:ty;)*) => {
        $(
            impl<D: Decoder> DeserializeUpdate<D> for $ty {
                fn deserialize_update<'p, 'de>(
                    &mut self,
                    d: AnyDecoder<'p, 'de, D>,
                    ctx: Context,
                ) -> anyhow::Result<()> {
                    if let Some(update) = <Option::<Self> as Deserialize<D>>::deserialize(d, ctx)? {
                        *self = update;
                    }
                    Ok(())
                }
            }
        )*
    };
}

derive_deserialize_update_for_option! {
    u8; u16; u32; u64; u128;
    i8; i16; i32; i64; i128;
    f32; f64;
    bool;
    char;
}
