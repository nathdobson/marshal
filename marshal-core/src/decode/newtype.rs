use crate::decode::{SimpleDecoderView, SpecDecoder};

pub fn cast_simple_decoder_view<
    'de,
    T1: SpecDecoder<'de>,
    T2: SpecDecoder<
        'de,
        AnyDecoder = T1::AnyDecoder,
        SomeDecoder = T1::SomeDecoder,
        SeqDecoder = T1::SeqDecoder,
        MapDecoder = T1::MapDecoder,
        DiscriminantDecoder = T1::DiscriminantDecoder,
    >,
>(
    x: SimpleDecoderView<'de, T1>,
) -> SimpleDecoderView<'de, T2> {
    match x {
        SimpleDecoderView::Primitive(x) => SimpleDecoderView::Primitive(x),
        SimpleDecoderView::String(x) => SimpleDecoderView::String(x),
        SimpleDecoderView::Bytes(x) => SimpleDecoderView::Bytes(x),
        SimpleDecoderView::None => SimpleDecoderView::None,
        SimpleDecoderView::Some(x) => SimpleDecoderView::Some(x),
        SimpleDecoderView::Seq(x) => SimpleDecoderView::Seq(x),
        SimpleDecoderView::Map(x) => SimpleDecoderView::Map(x),
        SimpleDecoderView::Enum(x) => SimpleDecoderView::Enum(x),
    }
}

#[macro_export]
macro_rules! derive_decoder_for_newtype {
    ($ty:ident <'de $(, $lt:lifetime )* > ($inner:ty)) =>{
        const _ : () = {
            use $crate::decode::DecodeHint;
            use $crate::decode::DecodeVariantHint;
            use $crate::decode::SpecDecoder;
            use $crate::decode::SimpleDecoderView;
            use $crate::decode::newtype::cast_simple_decoder_view;
            impl<'de $(, $lt)*> SpecDecoder<'de> for $ty <'de $(, $lt)*> {
                type AnyDecoder = <$inner as SpecDecoder<'de>>::AnyDecoder;
                type SeqDecoder = <$inner as SpecDecoder<'de>>::SeqDecoder;
                type MapDecoder = <$inner as SpecDecoder<'de>>::MapDecoder;
                type KeyDecoder = <$inner as SpecDecoder<'de>>::KeyDecoder;
                type ValueDecoder = <$inner as SpecDecoder<'de>>::ValueDecoder;
                type DiscriminantDecoder = <$inner as SpecDecoder<'de>>::DiscriminantDecoder;
                type VariantDecoder = <$inner as SpecDecoder<'de>>::VariantDecoder;
                type EnumCloser = <$inner as SpecDecoder<'de>>::EnumCloser;
                type SomeDecoder = <$inner as SpecDecoder<'de>>::SomeDecoder;
                type SomeCloser = <$inner as SpecDecoder<'de>>::SomeCloser;                #[inline]
                fn decode(
                    &mut self,
                    any: Self::AnyDecoder,
                    hint: DecodeHint,
                ) -> anyhow::Result<SimpleDecoderView<'de, Self>> {
                    Ok(cast_simple_decoder_view(self.0.decode(any, hint)?))
                }
                #[inline]
                fn is_human_readable(&self) -> bool {
                    self.0.is_human_readable()
                }
                #[inline]
                fn decode_seq_next(
                    &mut self,
                    seq: &mut Self::SeqDecoder,
                ) -> anyhow::Result<Option<Self::AnyDecoder>> {
                    self.0.decode_seq_next(seq)
                }
                #[inline]
                fn decode_seq_exact_size(&self, seq: &Self::SeqDecoder) -> Option<usize> {
                    self.0.decode_seq_exact_size(seq)
                }                #[inline]
                fn decode_seq_end(&mut self, seq: Self::SeqDecoder) -> anyhow::Result<()>{
                    self.0.decode_seq_end(seq)
                }                #[inline]
                fn decode_map_next(
                    &mut self,
                    map: &mut Self::MapDecoder,
                ) -> anyhow::Result<Option<Self::KeyDecoder>> {
                    self.0.decode_map_next(map)
                }                #[inline]
                fn decode_map_exact_size(&self, map: &Self::MapDecoder) -> Option<usize> {
                    self.0.decode_map_exact_size(map)
                }                #[inline]
                fn decode_map_end(&mut self, map: Self::MapDecoder) -> anyhow::Result<()> {
                    self.0.decode_map_end(map)
                }                #[inline]
                fn decode_entry_key(
                    &mut self,
                    key: Self::KeyDecoder,
                ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)> {
                    self.0.decode_entry_key(key)
                }
                #[inline]
                fn decode_entry_value(&mut self, value: Self::ValueDecoder)
                    -> anyhow::Result<Self::AnyDecoder> {
                    self.0.decode_entry_value(value)
                }
                #[inline]
                fn decode_enum_discriminant(
                    &mut self,
                    e: Self::DiscriminantDecoder,
                ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)> {
                    self.0.decode_enum_discriminant(e)
                }
                #[inline]
                fn decode_enum_variant(
                    &mut self,
                    e: Self::VariantDecoder,
                    hint: DecodeVariantHint,
                ) -> anyhow::Result<(SimpleDecoderView<'de, Self>, Self::EnumCloser)> {
                    let (view,closer)=self.0.decode_enum_variant(e,hint)?;
                    Ok((cast_simple_decoder_view(view),closer))
                }
                #[inline]
                fn decode_enum_end(&mut self, e: Self::EnumCloser) -> anyhow::Result<()> {
                    self.0.decode_enum_end(e)
                }
                #[inline]
                fn decode_some_inner(
                    &mut self,
                    e: Self::SomeDecoder,
                ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)> {
                    self.0.decode_some_inner(e)
                }
                #[inline]
                fn decode_some_end(&mut self, d: Self::SomeCloser) -> anyhow::Result<()> {
                    self.0.decode_some_end(d)
                }
            }
        };
    }
}
