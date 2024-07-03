use crate::decode::tuple_helper::TupleHelper;
use crate::decode::{AnySpecDecoder, DecodeHint, DecodeVariantHint, EnumDecoder, SpecDecoder};
use crate::SchemaError;

pub struct EnumDecoderHelper<'p, 'de, D: ?Sized + SpecDecoder<'de>> {
    variants: &'static [&'static str],
    decoder: EnumDecoder<'p, 'de, D>,
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> AnySpecDecoder<'p, 'de, D> {
    pub fn decode_enum_helper(
        self,
        name: &'static str,
        variants: &'static [&'static str],
    ) -> anyhow::Result<(usize, EnumDecoder<'p, 'de, D>)> {
        let mut d = self
            .decode(DecodeHint::Enum { name, variants })?
            .try_into_enum()?;
        let disc = d
            .decode_discriminant()?
            .decode(DecodeHint::Identifier)?
            .try_into_identifier(variants)?
            .ok_or(SchemaError::UnknownVariant)?;
        Ok((disc, d))
    }
}

impl<'p, 'de, D: ?Sized + SpecDecoder<'de>> EnumDecoder<'p, 'de, D> {
    pub fn decode_tuple_variant_helper<'p2>(
        &'p2 mut self,
        len: usize,
    ) -> anyhow::Result<TupleHelper<'p2, 'de, D>> {
        Ok(TupleHelper::new(
            self.decode_variant(DecodeVariantHint::TupleVariant { len })?
                .try_into_seq()?,
        ))
    }
}
