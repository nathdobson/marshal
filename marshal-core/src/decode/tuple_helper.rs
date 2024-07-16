use crate::decode::{AnySpecDecoder, SeqDecoder, SpecDecoder};
use crate::SchemaError;

pub struct TupleHelper<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    seq: SeqDecoder<'p, 'de, D>,
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> TupleHelper<'p, 'de, D> {
    #[inline]
    pub fn new(seq: SeqDecoder<'p, 'de, D>) -> Self {
        TupleHelper { seq }
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> TupleHelper<'p, 'de, D> {
    #[inline]
    pub fn decode_next<'p2>(&'p2 mut self) -> anyhow::Result<AnySpecDecoder<'p2, 'de, D>> {
        Ok(self.seq.decode_next()?.ok_or(SchemaError::TupleTooShort)?)
    }
    #[inline]
    pub fn decode_end(mut self, expected: usize) -> anyhow::Result<()> {
        if let Some(_) = self.seq.decode_next()? {
            Err(SchemaError::TupleTooLong { expected }.into())
        } else {
            Ok(())
        }
    }
}
