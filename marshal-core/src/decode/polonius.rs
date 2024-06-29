//! These operations work around limitations of the pre-polonius model checker

use crate::decode::{Decoder, EntryDecoder, MapDecoder};

impl<'p, D: ?Sized + Decoder> EntryDecoder<'p, D> {
    pub fn polonius(
        self,
    ) -> impl for<'p2, 'p3> FnOnce(&'p2 mut MapDecoder<'p3, D>) -> EntryDecoder<'p2, D> {
        let key = self.key;
        let value = self.value;
        |decoder| EntryDecoder {
            this: decoder.this,
            key,
            value,
        }
    }
}
