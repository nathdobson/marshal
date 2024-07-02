use crate::decode::{AnySpecDecoder, DecodeHint, DecoderView, EntryDecoder, MapDecoder, SpecDecoder};

pub struct StructDecoderHelper<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    fields: &'static [&'static str],
    decoder: MapDecoder<'p, 'de, D>,
}

pub struct FieldDecoderHelper<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    decoder: EntryDecoder<'p, 'de, D>,
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> AnySpecDecoder<'p, 'de, D> {
    pub fn decode_struct_helper(
        self,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<StructDecoderHelper<'p, 'de, D>> {
        let decoder = self
            .decode(DecodeHint::Struct { name, fields })?
            .try_into_map()?;
        Ok(StructDecoderHelper { fields, decoder })
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> StructDecoderHelper<'p, 'de, D> {
    pub fn next<'p2>(
        &'p2 mut self,
    ) -> anyhow::Result<Option<(usize, FieldDecoderHelper<'p2, 'de, D>)>> {
        loop {
            let d = self.decoder.decode_next()?;
            if let Some(mut d) = d {
                let n = match d.decode_key()?.decode(DecodeHint::Identifier)? {
                    DecoderView::Primitive(p) => {
                        let n: usize = p.try_into()?;
                        if n >= self.fields.len() {
                            d.decode_value()?.ignore()?;
                            d.decode_end()?;
                            continue;
                        }
                        n
                    }
                    DecoderView::String(s) => {
                        if let Some(n) = self.fields.iter().position(|x| **x == s) {
                            n
                        } else {
                            d.decode_value()?.ignore()?;
                            d.decode_end()?;
                            continue;
                        }
                    }
                    unexpected => unexpected.mismatch("identifier")?,
                };
                return Ok(Some((
                    n,
                    FieldDecoderHelper {
                        decoder: d.polonius()(&mut self.decoder),
                    },
                )));
            } else {
                return Ok(None);
            }
        }
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> FieldDecoderHelper<'p, 'de, D> {
    pub fn decode_field<'p2>(&'p2 mut self) -> anyhow::Result<AnySpecDecoder<'p2, 'de, D>> {
        self.decoder.decode_value()
    }
    pub fn decode_end(self) -> anyhow::Result<()> {
        self.decoder.decode_end()
    }
}
