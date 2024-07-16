use crate::decode::{AnySpecDecoder, DecodeHint, DecoderView, SpecDecoder};

pub struct StructDecoderHelper<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    fields: &'static [&'static str],
    decoder: &'p mut D,
    map: Option<D::MapDecoder>,
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> AnySpecDecoder<'p, 'de, D> {
    #[inline]
    pub fn decode_struct_helper(
        self,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<StructDecoderHelper<'p, 'de, D>> {
        let decoder = self
            .decode(DecodeHint::Struct { name, fields })?
            .try_into_map()?;
        let (decoder, map) = decoder.into_raw();
        Ok(StructDecoderHelper {
            fields,
            decoder,
            map: Some(map),
        })
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> StructDecoderHelper<'p, 'de, D> {
    #[inline]
    pub fn next<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<Option<(usize, AnySpecDecoder<'p2, 'de, D>)>> {
        loop {
            let map = if let Some(map) = self.map.as_mut() {
                map
            } else {
                return Ok(None);
            };
            if let Some(key) = self.decoder.decode_map_next(map)? {
                let (key, value) = self.decoder.decode_entry_key(key)?;
                let n = match AnySpecDecoder::<D>::new(self.decoder, key)
                    .decode(DecodeHint::Identifier)?
                {
                    DecoderView::Primitive(p) => {
                        let n: usize = p.try_into()?;
                        if n >= self.fields.len() {
                            let value = self.decoder.decode_entry_value(value)?;
                            AnySpecDecoder::new(self.decoder, value).ignore()?;
                            continue;
                        }
                        n
                    }
                    DecoderView::String(s) => {
                        if let Some(n) = self.fields.iter().position(|x| **x == s) {
                            n
                        } else {
                            let value = self.decoder.decode_entry_value(value)?;
                            AnySpecDecoder::new(self.decoder, value).ignore()?;
                            continue;
                        }
                    }
                    unexpected => unexpected.mismatch("identifier")?,
                };
                let value = self.decoder.decode_entry_value(value)?;
                return Ok(Some((n, AnySpecDecoder::new(self.decoder, value))));
            } else {
                self.decoder.decode_map_end(self.map.take().unwrap())?;
                return Ok(None);
            }
        }
    }
}