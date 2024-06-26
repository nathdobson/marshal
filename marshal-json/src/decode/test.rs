use std::fs;
use std::fs::read_dir;

use marshal::context::OwnedContext;
use marshal_core::{Primitive, PrimitiveType};
use marshal_core::decode::{AnySpecDecoder, DecodeHint, DecoderView};

use crate::decode::{JsonAnyDecoder, SimpleJsonSpecDecoder};
use crate::decode::full::JsonDecoderBuilder;
use crate::value::JsonValue;

#[test]
fn test() -> anyhow::Result<()> {
    let input = b"[1,23]";
    let mut p = SimpleJsonSpecDecoder::new(input);
    let p = AnySpecDecoder::new(&mut p, JsonAnyDecoder::default());
    match p.decode(DecodeHint::Any)? {
        DecoderView::Seq(mut p) => {
            match p
                .decode_next()?
                .unwrap()
                .decode(DecodeHint::Primitive(PrimitiveType::U64))?
            {
                DecoderView::Primitive(Primitive::U64(x)) => assert_eq!(x, 1),
                _ => todo!(),
            }
            match p
                .decode_next()?
                .unwrap()
                .decode(DecodeHint::Primitive(PrimitiveType::U64))?
            {
                DecoderView::Primitive(Primitive::U64(x)) => assert_eq!(x, 23),
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
    Ok(())
}

#[test]
fn test_parsing() {
    for dir in read_dir("../JSONTestSuite/test_parsing").unwrap() {
        let dir = dir.unwrap();
        let expected = dir
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .next()
            .unwrap();
        let contents = fs::read(dir.path()).unwrap();
        let output = JsonDecoderBuilder::new(&contents)
            .deserialize::<JsonValue>(OwnedContext::new().borrow());
        match expected {
            'i' => {}
            'n' => {
                assert!(output.is_err());
            }
            'y' => {
                output.unwrap();
            }
            _ => unreachable!(),
        }
    }
}
