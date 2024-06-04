use crate::Primitive;

pub trait Writer: Sized {
    type AnyWriter<'w>: AnyWriter<'w, Self>
    where
        Self: 'w;
    type SomeWriter<'w>: SomeWriter<'w, Self>
    where
        Self: 'w;
}

pub trait AnyWriter<'w, W: Writer> {
    fn serialize_prim(self, prim: Primitive) -> anyhow::Result<()>;
    fn serialize_none(self) -> anyhow::Result<()>;
    fn serialize_some(self) -> anyhow::Result<<W as Writer>::SomeWriter<'w>>;
}

pub trait SomeWriter<'w, W: Writer> {
    fn write_some<'w2>(&'w2 mut self) -> anyhow::Result<<W as Writer>::AnyWriter<'w2>>;
    fn end(self) -> anyhow::Result<()>;
}

