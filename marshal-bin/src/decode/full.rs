use crate::decode::{BinAnyDecoder, BinDecoderSchema, SimpleBinDecoder};
use marshal_core::decode::depth_budget::{DepthBudgetDecoder, WithDepthBudget};
use marshal_core::decode::poison::{PoisonAnyDecoder, PoisonDecoder, PoisonState};
use marshal_core::decode::simple::{SimpleAnyDecoder, SimpleDecoderAdapter};
use marshal_core::decode::Decoder;

pub type BinDecoder<'de, 's> =
PoisonDecoder<DepthBudgetDecoder<SimpleDecoderAdapter<SimpleBinDecoder<'de, 's>>>>;

pub struct BinDecoderBuilder<'de, 's> {
    inner: SimpleBinDecoder<'de, 's>,
    depth_budget: usize,
    poison: PoisonState,
}

impl<'de, 's> BinDecoderBuilder<'de, 's> {
    pub fn new(input: &'de [u8], schema: &'s mut BinDecoderSchema) -> Self {
        BinDecoderBuilder {
            inner: SimpleBinDecoder::new(input, schema),
            depth_budget: 100,
            poison: PoisonState::new(),
        }
    }
    pub fn build<'p>(&'p mut self) -> <BinDecoder<'de, 's> as Decoder<'de>>::AnyDecoder<'p> {
        PoisonAnyDecoder::new(
            &mut self.poison,
            WithDepthBudget::new(
                self.depth_budget,
                SimpleAnyDecoder::new(&mut self.inner, BinAnyDecoder::Read),
            ),
        )
    }
    pub fn end(self) -> anyhow::Result<()> {
        self.poison.check()?;
        self.inner.end()
    }
}
