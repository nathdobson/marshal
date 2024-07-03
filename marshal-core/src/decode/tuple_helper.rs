use crate::decode::{AnySpecDecoder, SeqDecoder, SpecDecoder};
use crate::SchemaError;

pub struct TupleHelper<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    seq: SeqDecoder<'p, 'de, D>,
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> TupleHelper<'p, 'de, D> {
    pub fn new(seq: SeqDecoder<'p, 'de, D>) -> Self {
        TupleHelper { seq }
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> TupleHelper<'p, 'de, D> {
    pub fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<AnySpecDecoder<'p2, 'de, D>> {
        Ok(self.seq.decode_next()?.ok_or(SchemaError::TupleTooShort)?)
    }
    pub fn decode_end(mut self) -> anyhow::Result<()> {
        if let Some(_) = self.seq.decode_next()? {
            Err(SchemaError::TupleTooLong.into())
        } else {
            Ok(())
        }
    }
}
