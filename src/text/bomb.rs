use std::ops::{Deref, DerefMut};

use crate::text::TextParser;

pub struct TextBomb<'p, 'de> {
    parser: Option<&'p mut TextParser<'de>>,
}

impl<'p, 'de> TextBomb<'p, 'de> {
    pub fn new(parser: &'p mut TextParser<'de>) -> Self {
        TextBomb {
            parser: Some(parser),
        }
    }
    pub fn into_inner(mut self) -> &'p mut TextParser<'de> {
        self.parser.take().unwrap()
    }
    pub fn defuse(&mut self) {
        self.parser = None;
    }
}

impl<'p, 'de> Deref for TextBomb<'p, 'de> {
    type Target = TextParser<'de>;
    fn deref(&self) -> &Self::Target {
        self.parser.as_ref().unwrap()
    }
}

impl<'p, 'de> DerefMut for TextBomb<'p, 'de> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.parser.as_mut().unwrap()
    }
}

impl<'p, 'de> Drop for TextBomb<'p, 'de> {
    fn drop(&mut self) {
        if let Some(parser) = self.parser.as_mut() {
            parser.poisoned = true;
        }
    }
}