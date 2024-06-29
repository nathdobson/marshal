use crate::decode::{Decoder, SimpleDecoderView};

pub fn cast_simple_decoder_view<
    T1: Decoder,
    T2: Decoder<
        AnyDecoder = T1::AnyDecoder,
        StringDecoder = T1::StringDecoder,
        BytesDecoder = T1::BytesDecoder,
        SomeDecoder = T1::SomeDecoder,
        SeqDecoder = T1::SeqDecoder,
        MapDecoder = T1::MapDecoder,
        DiscriminantDecoder = T1::DiscriminantDecoder,
    >,
>(
    x: SimpleDecoderView<T1>,
) -> SimpleDecoderView<T2> {
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
    ($ty:ident < $(, $lt:lifetime )* > ($inner:ty)) =>{
        const _ : () = {
            use $crate::decode::DecodeHint;
            use $crate::decode::DecodeVariantHint;
            use $crate::decode::Decoder;
            use $crate::decode::SimpleDecoderView;
            use $crate::decode::newtype::cast_simple_decoder_view;
            impl< $(, $lt)*> Decoder for $ty < $(, $lt)*> {
                type AnyDecoder = <$inner as Decoder>::AnyDecoder;
                type StringDecoder = <$inner as Decoder>::StringDecoder;
                type BytesDecoder = <$inner as Decoder>::BytesDecoder;
                type SeqDecoder = <$inner as Decoder>::SeqDecoder;
                type MapDecoder = <$inner as Decoder>::MapDecoder;
                type KeyDecoder = <$inner as Decoder>::KeyDecoder;
                type ValueDecoder = <$inner as Decoder>::ValueDecoder;
                type DiscriminantDecoder = <$inner as Decoder>::DiscriminantDecoder;
                type VariantDecoder = <$inner as Decoder>::VariantDecoder;
                type EnumCloser = <$inner as Decoder>::EnumCloser;
                type SomeDecoder = <$inner as Decoder>::SomeDecoder;
                type SomeCloser = <$inner as Decoder>::SomeCloser;
                fn decode(
                    &mut self,
                    any: Self::AnyDecoder,
                    hint: DecodeHint,
                ) -> anyhow::Result<SimpleDecoderView< Self>> {
                    Ok(cast_simple_decoder_view(self.0.decode(any, hint)?))
                }

                fn is_human_readable(&self) -> bool {
                    self.0.is_human_readable()
                }

                fn decode_seq_next(
                    &mut self,
                    seq: &mut Self::SeqDecoder,
                ) -> anyhow::Result<Option<Self::AnyDecoder>> {
                    self.0.decode_seq_next(seq)
                }

                fn decode_seq_exact_size(&self, seq: &Self::SeqDecoder) -> Option<usize> {
                    self.0.decode_seq_exact_size(seq)
                }
                fn decode_seq_end(&mut self, seq: Self::SeqDecoder) -> anyhow::Result<()>{
                    self.0.decode_seq_end(seq)
                }
                fn decode_map_next(
                    &mut self,
                    map: &mut Self::MapDecoder,
                ) -> anyhow::Result<Option<Self::KeyDecoder>> {
                    self.0.decode_map_next(map)
                }
                fn decode_map_exact_size(&self, map: &Self::MapDecoder) -> Option<usize> {
                    self.0.decode_map_exact_size(map)
                }
                fn decode_map_end(&mut self, map: Self::MapDecoder) -> anyhow::Result<()> {
                    self.0.decode_map_end(map)
                }
                fn decode_entry_key(
                    &mut self,
                    key: Self::KeyDecoder,
                ) -> anyhow::Result<(Self::AnyDecoder, Self::ValueDecoder)> {
                    self.0.decode_entry_key(key)
                }

                fn decode_entry_value(&mut self, value: Self::ValueDecoder)
                    -> anyhow::Result<Self::AnyDecoder> {
                    self.0.decode_entry_value(value)
                }

                fn decode_enum_discriminant(
                    &mut self,
                    e: Self::DiscriminantDecoder,
                ) -> anyhow::Result<(Self::AnyDecoder, Self::VariantDecoder)> {
                    self.0.decode_enum_discriminant(e)
                }

                fn decode_enum_variant(
                    &mut self,
                    e: Self::VariantDecoder,
                    hint: DecodeVariantHint,
                ) -> anyhow::Result<(SimpleDecoderView< Self>, Self::EnumCloser)> {
                    let (view,closer)=self.0.decode_enum_variant(e,hint)?;
                    Ok((cast_simple_decoder_view(view),closer))
                }

                fn decode_enum_end(&mut self, e: Self::EnumCloser) -> anyhow::Result<()> {
                    self.0.decode_enum_end(e)
                }

                fn decode_some_inner(
                    &mut self,
                    e: Self::SomeDecoder,
                ) -> anyhow::Result<(Self::AnyDecoder, Self::SomeCloser)> {
                    self.0.decode_some_inner(e)
                }

                fn decode_some_end(&mut self, d: Self::SomeCloser) -> anyhow::Result<()> {
                    self.0.decode_some_end(d)
                }

                fn decode_string_cow(&mut self, d: Self::StringDecoder) -> anyhow::Result<::std::borrow::Cow<str>> {
                    self.0.decode_string_cow(d)
                }

                fn decode_bytes_cow(&mut self, d: Self::BytesDecoder) -> anyhow::Result<::std::borrow::Cow<[u8]>> {
                    self.0.decode_bytes_cow(d)
                }
            }
        };
    }
}
