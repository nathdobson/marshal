#[macro_export]
macro_rules! derive_encoder_for_newtype {
    ($ty:ident $(<$($lt:lifetime ),* >)? ($inner:ty)) =>{
        const _ : () = {
            use $crate::encode::SpecEncoder;
            use $crate::Primitive;
            impl $(<$( $lt),*>)? SpecEncoder for $ty $(<$( $lt),*>)? {
                type AnySpecEncoder = <$inner as SpecEncoder>::AnySpecEncoder;
                type SomeCloser = <$inner as SpecEncoder>::SomeCloser;
                type TupleEncoder = <$inner as SpecEncoder>::TupleEncoder;
                type SeqEncoder = <$inner as SpecEncoder>::SeqEncoder;
                type MapEncoder = <$inner as SpecEncoder>::MapEncoder;
                type ValueEncoder = <$inner as SpecEncoder>::ValueEncoder;
                type EntryCloser = <$inner as SpecEncoder>::EntryCloser;
                type TupleStructEncoder = <$inner as SpecEncoder>::TupleStructEncoder;
                type StructEncoder = <$inner as SpecEncoder>::StructEncoder;
                type TupleVariantEncoder = <$inner as SpecEncoder>::TupleVariantEncoder;
                type StructVariantEncoder = <$inner as SpecEncoder>::StructVariantEncoder;

                #[inline]
                fn encode_prim(&mut self, any: Self::AnySpecEncoder, prim: Primitive) -> anyhow::Result<()> {
                    self.0.encode_prim(any, prim)
                }

                #[inline]
                fn encode_str(&mut self, any: Self::AnySpecEncoder, s: &str) -> anyhow::Result<()> {
                    self.0.encode_str(any, s)
                }

                #[inline]
                fn encode_bytes(&mut self, any: Self::AnySpecEncoder, s: &[u8]) -> anyhow::Result<()> {
                    self.0.encode_bytes(any, s)
                }

                #[inline]
                fn encode_none(&mut self, any: Self::AnySpecEncoder) -> anyhow::Result<()> {
                    self.0.encode_none(any)
                }

                #[inline]
                fn encode_some(
                    &mut self,
                    any: Self::AnySpecEncoder,
                ) -> anyhow::Result<(Self::AnySpecEncoder, Self::SomeCloser)> {
                    self.0.encode_some(any)
                }

                #[inline]
                fn encode_unit_struct(
                    &mut self,
                    any: Self::AnySpecEncoder,
                    name: &'static str,
                ) -> anyhow::Result<()> {
                    self.0.encode_unit_struct(any, name)
                }

                #[inline]
                fn encode_tuple_struct(
                    &mut self,
                    any: Self::AnySpecEncoder,
                    name: &'static str,
                    len: usize,
                ) -> anyhow::Result<Self::TupleStructEncoder> {
                    self.0.encode_tuple_struct(any, name, len)
                }

                #[inline]
                fn encode_struct(
                    &mut self,
                    any: Self::AnySpecEncoder,
                    name: &'static str,
                    fields: &'static [&'static str],
                ) -> anyhow::Result<Self::StructEncoder> {
                    self.0.encode_struct(any, name, fields)
                }

                #[inline]
                fn encode_unit_variant(
                    &mut self,
                    any: Self::AnySpecEncoder,
                    name: &'static str,
                    variants: &'static [&'static str],
                    variant_index: usize,
                ) -> anyhow::Result<()> {
                    self.0
                        .encode_unit_variant(any, name, variants, variant_index)
                }

                #[inline]
                fn encode_tuple_variant(
                    &mut self,
                    any: Self::AnySpecEncoder,
                    name: &'static str,
                    variants: &'static [&'static str],
                    variant_index: usize,
                    len: usize,
                ) -> anyhow::Result<Self::TupleVariantEncoder> {
                    self.0
                        .encode_tuple_variant(any, name, variants, variant_index, len)
                }

                #[inline]
                fn encode_struct_variant(
                    &mut self,
                    any: Self::AnySpecEncoder,
                    name: &'static str,
                    variants: &'static [&'static str],
                    variant_index: usize,
                    fields: &'static [&'static str],
                ) -> anyhow::Result<Self::StructVariantEncoder> {
                    self.0
                        .encode_struct_variant(any, name, variants, variant_index, fields)
                }

                #[inline]
                fn encode_seq(
                    &mut self,
                    any: Self::AnySpecEncoder,
                    len: Option<usize>,
                ) -> anyhow::Result<Self::SeqEncoder> {
                    self.0.encode_seq(any, len)
                }

                #[inline]
                fn encode_tuple(
                    &mut self,
                    any: Self::AnySpecEncoder,
                    len: usize,
                ) -> anyhow::Result<Self::TupleEncoder> {
                    self.0.encode_tuple(any, len)
                }

                #[inline]
                fn encode_map(
                    &mut self,
                    any: Self::AnySpecEncoder,
                    len: Option<usize>,
                ) -> anyhow::Result<Self::MapEncoder> {
                    self.0.encode_map(any, len)
                }

                #[inline]
                fn some_end(&mut self, some: Self::SomeCloser) -> anyhow::Result<()> {
                    self.0.some_end(some)
                }

                #[inline]
                fn tuple_encode_element(
                    &mut self,
                    tuple: &mut Self::TupleEncoder,
                ) -> anyhow::Result<Self::AnySpecEncoder> {
                    self.0.tuple_encode_element(tuple)
                }

                #[inline]
                fn tuple_end(&mut self, tuple: Self::TupleEncoder) -> anyhow::Result<()> {
                    self.0.tuple_end(tuple)
                }

                #[inline]
                fn seq_encode_element(
                    &mut self,
                    seq: &mut Self::SeqEncoder,
                ) -> anyhow::Result<Self::AnySpecEncoder> {
                    self.0.seq_encode_element(seq)
                }

                #[inline]
                fn seq_end(&mut self, tuple: Self::SeqEncoder) -> anyhow::Result<()> {
                    self.0.seq_end(tuple)
                }

                #[inline]
                fn map_encode_element(
                    &mut self,
                    map: &mut Self::MapEncoder,
                ) -> anyhow::Result<(Self::AnySpecEncoder, Self::ValueEncoder)> {
                    self.0.map_encode_element(map)
                }

                #[inline]
                fn map_end(&mut self, map: Self::MapEncoder) -> anyhow::Result<()> {
                    self.0.map_end(map)
                }

                #[inline]
                fn entry_encode_value(
                    &mut self,
                    value: Self::ValueEncoder,
                ) -> anyhow::Result<(Self::AnySpecEncoder, Self::EntryCloser)> {
                    self.0.entry_encode_value(value)
                }

                #[inline]
                fn entry_end(&mut self, closer: Self::EntryCloser) -> anyhow::Result<()> {
                    self.0.entry_end(closer)
                }

                #[inline]
                fn tuple_struct_encode_field(
                    &mut self,
                    map: &mut Self::TupleStructEncoder,
                ) -> anyhow::Result<Self::AnySpecEncoder> {
                    self.0.tuple_struct_encode_field(map)
                }

                #[inline]
                fn tuple_struct_end(&mut self, map: Self::TupleStructEncoder) -> anyhow::Result<()> {
                    self.0.tuple_struct_end(map)
                }

                #[inline]
                fn struct_encode_field(
                    &mut self,
                    map: &mut Self::StructEncoder,
                    field: &'static str,
                ) -> anyhow::Result<Self::AnySpecEncoder> {
                    self.0.struct_encode_field(map, field)
                }

                #[inline]
                fn struct_end(&mut self, map: Self::StructEncoder) -> anyhow::Result<()> {
                    self.0.struct_end(map)
                }

                #[inline]
                fn tuple_variant_encode_field(
                    &mut self,
                    map: &mut Self::TupleVariantEncoder,
                ) -> anyhow::Result<Self::AnySpecEncoder> {
                    self.0.tuple_variant_encode_field(map)
                }

                #[inline]
                fn tuple_variant_end(&mut self, map: Self::TupleVariantEncoder) -> anyhow::Result<()> {
                    self.0.tuple_variant_end(map)
                }

                #[inline]
                fn struct_variant_encode_field(
                    &mut self,
                    map: &mut Self::StructVariantEncoder,
                    key: &'static str,
                ) -> anyhow::Result<Self::AnySpecEncoder> {
                    self.0.struct_variant_encode_field(map, key)
                }

                #[inline]
                fn struct_variant_end(&mut self, map: Self::StructVariantEncoder) -> anyhow::Result<()> {
                    self.0.struct_variant_end(map)
                }

                #[inline]
                fn is_human_readable(&self) -> bool {
                    self.0.is_human_readable()
                }
            }
        };
    }
}
