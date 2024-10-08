use marshal::context::Context;
use marshal::encode::AnyEncoder;
use marshal::encode::Encoder;
use marshal::ser::Serialize;

use crate::ser::SerializeStream;
use crate::ser::SerializeUpdate;

macro_rules! derive_serialize_update_for_clone_eq {
    ($($ty:ty;)*) => {
        $(
            impl SerializeStream for $ty {
                type Stream = $ty;
                fn start_stream(&self, _ctx: Context) -> anyhow::Result<Self::Stream> {
                    println!("starting stream");
                    Ok(self.clone())
                }
            }
            impl<E: Encoder> SerializeUpdate<E> for $ty {
                fn serialize_update<'w,'en>(
                    &self,
                    stream: &mut Self::Stream,
                    e: AnyEncoder<'w,'en,E>,
                    ctx: Context,
                ) -> anyhow::Result<()> {
                    let m = if stream != self {
                        stream.clone_from(self);
                        println!("A");
                        Some(&self)
                    } else {
                        println!("B");
                        None
                    };
                    <Option<&&$ty> as Serialize<E>>::serialize(&m, e, ctx)?;
                    Ok(())
                }
            }
        )*
    };
}

derive_serialize_update_for_clone_eq! {
    u8; u16; u32; u64; u128;
    i8; i16; i32; i64; i128;
    f32; f64;
    bool;
    char;
}
