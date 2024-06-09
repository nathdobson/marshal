use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal_bin::encode::full::BinEncoder;
use marshal_json::encode::full::{JsonEncoder, JsonEncoderBuilder};

trait SerializeJson = Serialize<JsonEncoder>;
trait SerializeBin = for<'s> Serialize<BinEncoder<'s>>;

trait DynSerialize<Tr: ?Sized> {
    fn variant_index(&self) -> usize;
}
trait MyTrait: SerializeJson + SerializeBin + DynSerialize<dyn MyTrait> {}

impl<E: Encoder> Serialize<E> for Box<dyn MyTrait>
where
    dyn MyTrait: Serialize<E>,
{
    fn serialize(&self, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
        let e = e.encode_tuple_variant("dyn MyTrait", &["u8", "u16"], self.variant_index(), 1)?;
    }
}

impl DynSerialize<dyn MyTrait> for u8 {
    fn variant_index(&self) -> usize {
        0
    }
}

impl DynSerialize<dyn MyTrait> for u16 {
    fn variant_index(&self) -> usize {
        1
    }
}

impl MyTrait for u8 {}
impl MyTrait for u16 {}

#[test]
fn test() -> anyhow::Result<()> {
    let encoded = JsonEncoderBuilder::new()
        .serialize(&(Box::new(42u8) as Box<dyn MyTrait>), &mut Context::new())?;
    assert_eq!(encoded, "");
    Ok(())
}
