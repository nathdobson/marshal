use crate::read::{BinAnyParser, BinParserSchema, SimpleBinParser};
use marshal_core::parse::depth_budget::{DepthBudgetParser, WithDepthBudget};
use marshal_core::parse::poison::{PoisonAnyParser, PoisonParser, PoisonState};
use marshal_core::parse::simple::{SimpleAnyParser, SimpleParserAdapter};
use marshal_core::parse::Parser;

pub type BinParser<'de, 's> =
    PoisonParser<DepthBudgetParser<SimpleParserAdapter<SimpleBinParser<'de, 's>>>>;

pub struct BinParserBuilder<'de, 's> {
    inner: SimpleBinParser<'de, 's>,
    depth_budget: usize,
    poison: PoisonState,
}

impl<'de, 's> BinParserBuilder<'de, 's> {
    pub fn new(input: &'de [u8], schema: &'s mut BinParserSchema) -> Self {
        BinParserBuilder {
            inner: SimpleBinParser::new(input, schema),
            depth_budget: 100,
            poison: PoisonState::new(),
        }
    }
    pub fn build<'p>(&'p mut self) -> <BinParser<'de, 's> as Parser<'de>>::AnyParser<'p> {
        PoisonAnyParser::new(
            &mut self.poison,
            WithDepthBudget::new(
                self.depth_budget,
                SimpleAnyParser::new(&mut self.inner, BinAnyParser::Read),
            ),
        )
    }
    pub fn end(self) -> anyhow::Result<()> {
        self.poison.check()?;
        self.inner.end()
    }
}
