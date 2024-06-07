use crate::read::{BinAnyParser, BinParserSchema, SimpleBinParser};
use crate::write::SimpleBinWriter;
use marshal_core::parse::simple::{SimpleAnyParser, SimpleParserAdapter};
use marshal_core::parse::Parser;

pub type BinParser<'de, 's> = SimpleParserAdapter<SimpleBinParser<'de, 's>>;

pub struct BinParserBuilder<'de, 's> {
    inner: SimpleBinParser<'de, 's>,
}

impl<'de, 's> BinParserBuilder<'de, 's> {
    pub fn new(input: &'de [u8], schema: &'s mut BinParserSchema) -> Self {
        BinParserBuilder {
            inner: SimpleBinParser::new(input, schema),
        }
    }
    pub fn build<'p>(&'p mut self) -> <BinParser<'de, 's> as Parser<'de>>::AnyParser<'p> {
        SimpleAnyParser::new(&mut self.inner, BinAnyParser::Read)
    }
    pub fn end(self) -> anyhow::Result<()> {
        self.inner.end()
    }
}
