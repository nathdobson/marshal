
use crate::write::Writer;

pub trait Serialize<W: Writer> {
    fn serialize(w: W::AnyWriter<'_>) -> anyhow::Result<()>;
}
