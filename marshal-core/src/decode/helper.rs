use crate::decode::{AnyDecoder, DecodeHint, Decoder, DecoderView, EntryDecoder, MapDecoder};

pub struct StructDecoderHelper<'p, D: ?Sized + Decoder> {
    fields: &'static [&'static str],
    decoder: MapDecoder<'p, D>,
}

pub struct FieldDecoderHelper<'p, D: ?Sized + Decoder> {
    decoder: EntryDecoder<'p, D>,
}

impl<'p, D: ?Sized + Decoder> AnyDecoder<'p, D> {
    pub fn decode_struct_helper(
        self,
        name: &'static str,
        fields: &'static [&'static str],
    ) -> anyhow::Result<StructDecoderHelper<'p, D>> {
        let decoder = self
            .decode(DecodeHint::Struct { name, fields })?
            .try_into_map()?;
        Ok(StructDecoderHelper { fields, decoder })
    }
}

impl<'p, D: ?Sized + Decoder> StructDecoderHelper<'p, D> {
    pub fn next<'p2>(&'p2 mut self) -> anyhow::Result<Option<(usize, FieldDecoderHelper<'p2, D>)>> {
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
                        let s = s.decode_cow()?;
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

impl<'p, D: Decoder> FieldDecoderHelper<'p, D> {
    pub fn decode_field<'p2>(&'p2 mut self) -> anyhow::Result<AnyDecoder<'p2, D>> {
        self.decoder.decode_value()
    }
    pub fn decode_end(self) -> anyhow::Result<()> {
        self.decoder.decode_end()
    }
}
