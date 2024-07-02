//! These operations work around limitations of the pre-polonius model checker

use crate::decode::{EntryDecoder, MapDecoder, SpecDecoder};

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> EntryDecoder<'p, 'de, D> {
    pub fn polonius(
        self,
    ) -> impl for<'p2, 'p3> FnOnce(&'p2 mut MapDecoder<'p3, 'de, D>) -> EntryDecoder<'p2, 'de, D>
    {
        let key = self.key;
        let value = self.value;
        |decoder| EntryDecoder {
            this: decoder.this,
            key,
            value,
        }
    }
}
