#[macro_export]
macro_rules! derive_encoder_for_newtype {
    ($ty:ident $(<$($lt:lifetime ),* >)? ($inner:ty)) =>{
        const _ : () = {
            use $crate::encode::Encoder;
            use $crate::Primitive;
            impl $(<$( $lt),*>)? Encoder for $ty $(<$( $lt),*>)? {
                type AnyEncoder = <$inner as Encoder>::AnyEncoder;
                type SomeCloser = <$inner as Encoder>::SomeCloser;
                type TupleEncoder = <$inner as Encoder>::TupleEncoder;
                type SeqEncoder = <$inner as Encoder>::SeqEncoder;
                type MapEncoder = <$inner as Encoder>::MapEncoder;
                type ValueEncoder = <$inner as Encoder>::ValueEncoder;
                type EntryCloser = <$inner as Encoder>::EntryCloser;
                type TupleStructEncoder = <$inner as Encoder>::TupleStructEncoder;
                type StructEncoder = <$inner as Encoder>::StructEncoder;
                type TupleVariantEncoder = <$inner as Encoder>::TupleVariantEncoder;
                type StructVariantEncoder = <$inner as Encoder>::StructVariantEncoder;

                fn encode_prim(&mut self, any: Self::AnyEncoder, prim: Primitive) -> anyhow::Result<()> {
                    self.0.encode_prim(any, prim)
                }

                fn encode_str(&mut self, any: Self::AnyEncoder, s: &str) -> anyhow::Result<()> {
                    self.0.encode_str(any, s)
                }

                fn encode_bytes(&mut self, any: Self::AnyEncoder, s: &[u8]) -> anyhow::Result<()> {
                    self.0.encode_bytes(any, s)
                }

                fn encode_none(&mut self, any: Self::AnyEncoder) -> anyhow::Result<()> {
                    self.0.encode_none(any)
                }

                fn encode_some(
                    &mut self,
                    any: Self::AnyEncoder,
                ) -> anyhow::Result<(Self::AnyEncoder, Self::SomeCloser)> {
                    self.0.encode_some(any)
                }

                fn encode_unit_struct(
                    &mut self,
                    any: Self::AnyEncoder,
                    name: &'static str,
                ) -> anyhow::Result<()> {
                    self.0.encode_unit_struct(any, name)
                }

                fn encode_tuple_struct(
                    &mut self,
                    any: Self::AnyEncoder,
                    name: &'static str,
                    len: usize,
                ) -> anyhow::Result<Self::TupleStructEncoder> {
                    self.0.encode_tuple_struct(any, name, len)
                }

                fn encode_struct(
                    &mut self,
                    any: Self::AnyEncoder,
                    name: &'static str,
                    fields: &'static [&'static str],
                ) -> anyhow::Result<Self::StructEncoder> {
                    self.0.encode_struct(any, name, fields)
                }

                fn encode_unit_variant(
                    &mut self,
                    any: Self::AnyEncoder,
                    name: &'static str,
                    variants: &'static [&'static str],
                    variant_index: usize,
                ) -> anyhow::Result<()> {
                    self.0
                        .encode_unit_variant(any, name, variants, variant_index)
                }

                fn encode_tuple_variant(
                    &mut self,
                    any: Self::AnyEncoder,
                    name: &'static str,
                    variants: &'static [&'static str],
                    variant_index: usize,
                    len: usize,
                ) -> anyhow::Result<Self::TupleVariantEncoder> {
                    self.0
                        .encode_tuple_variant(any, name, variants, variant_index, len)
                }

                fn encode_struct_variant(
                    &mut self,
                    any: Self::AnyEncoder,
                    name: &'static str,
                    variants: &'static [&'static str],
                    variant_index: usize,
                    fields: &'static [&'static str],
                ) -> anyhow::Result<Self::StructVariantEncoder> {
                    self.0
                        .encode_struct_variant(any, name, variants, variant_index, fields)
                }

                fn encode_seq(
                    &mut self,
                    any: Self::AnyEncoder,
                    len: Option<usize>,
                ) -> anyhow::Result<Self::SeqEncoder> {
                    self.0.encode_seq(any, len)
                }

                fn encode_tuple(
                    &mut self,
                    any: Self::AnyEncoder,
                    len: usize,
                ) -> anyhow::Result<Self::TupleEncoder> {
                    self.0.encode_tuple(any, len)
                }

                fn encode_map(
                    &mut self,
                    any: Self::AnyEncoder,
                    len: Option<usize>,
                ) -> anyhow::Result<Self::MapEncoder> {
                    self.0.encode_map(any, len)
                }

                fn some_end(&mut self, some: Self::SomeCloser) -> anyhow::Result<()> {
                    self.0.some_end(some)
                }

                fn tuple_encode_element(
                    &mut self,
                    tuple: &mut Self::TupleEncoder,
                ) -> anyhow::Result<Self::AnyEncoder> {
                    self.0.tuple_encode_element(tuple)
                }

                fn tuple_end(&mut self, tuple: Self::TupleEncoder) -> anyhow::Result<()> {
                    self.0.tuple_end(tuple)
                }

                fn seq_encode_element(
                    &mut self,
                    seq: &mut Self::SeqEncoder,
                ) -> anyhow::Result<Self::AnyEncoder> {
                    self.0.seq_encode_element(seq)
                }

                fn seq_end(&mut self, tuple: Self::SeqEncoder) -> anyhow::Result<()> {
                    self.0.seq_end(tuple)
                }

                fn map_encode_element(
                    &mut self,
                    map: &mut Self::MapEncoder,
                ) -> anyhow::Result<(Self::AnyEncoder, Self::ValueEncoder)> {
                    self.0.map_encode_element(map)
                }

                fn map_end(&mut self, map: Self::MapEncoder) -> anyhow::Result<()> {
                    self.0.map_end(map)
                }

                fn entry_encode_value(
                    &mut self,
                    value: Self::ValueEncoder,
                ) -> anyhow::Result<(Self::AnyEncoder, Self::EntryCloser)> {
                    self.0.entry_encode_value(value)
                }

                fn entry_end(&mut self, closer: Self::EntryCloser) -> anyhow::Result<()> {
                    self.0.entry_end(closer)
                }

                fn tuple_struct_encode_field(
                    &mut self,
                    map: &mut Self::TupleStructEncoder,
                ) -> anyhow::Result<Self::AnyEncoder> {
                    self.0.tuple_struct_encode_field(map)
                }

                fn tuple_struct_end(&mut self, map: Self::TupleStructEncoder) -> anyhow::Result<()> {
                    self.0.tuple_struct_end(map)
                }

                fn struct_encode_field(
                    &mut self,
                    map: &mut Self::StructEncoder,
                    field: &'static str,
                ) -> anyhow::Result<Self::AnyEncoder> {
                    self.0.struct_encode_field(map, field)
                }

                fn struct_end(&mut self, map: Self::StructEncoder) -> anyhow::Result<()> {
                    self.0.struct_end(map)
                }

                fn tuple_variant_encode_field(
                    &mut self,
                    map: &mut Self::TupleVariantEncoder,
                ) -> anyhow::Result<Self::AnyEncoder> {
                    self.0.tuple_variant_encode_field(map)
                }

                fn tuple_variant_end(&mut self, map: Self::TupleVariantEncoder) -> anyhow::Result<()> {
                    self.0.tuple_variant_end(map)
                }

                fn struct_variant_encode_field(
                    &mut self,
                    map: &mut Self::StructVariantEncoder,
                    key: &'static str,
                ) -> anyhow::Result<Self::AnyEncoder> {
                    self.0.struct_variant_encode_field(map, key)
                }

                fn struct_variant_end(&mut self, map: Self::StructVariantEncoder) -> anyhow::Result<()> {
                    self.0.struct_variant_end(map)
                }
            }
        };
    }
}
