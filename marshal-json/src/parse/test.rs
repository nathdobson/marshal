use std::fs;
use std::fs::read_dir;

use marshal::context::Context;
use marshal_core::{Primitive, PrimitiveType};
use marshal_core::decode::{AnyDecoder, DecodeHint, DecoderView, SeqDecoder};
use marshal_core::decode::simple::SimpleAnyParser;

use crate::parse::{JsonAnyParser, SimpleJsonParser};
use crate::parse::full::parse_json;
use crate::value::JsonValue;

#[test]
fn test() -> anyhow::Result<()> {
    let input = b"[1,23]";
    let mut p = SimpleJsonParser::new(input);
    let p = SimpleAnyParser::new(&mut p, JsonAnyParser::default());
    match p.decode(DecodeHint::Any)? {
        DecoderView::Seq(mut p) => {
            match p.decode_next()?.unwrap().decode(DecodeHint::Primitive(PrimitiveType::U64))? {
                DecoderView::Primitive(Primitive::U64(x)) => assert_eq!(x, 1),
                _ => todo!(),
            }
            match p.decode_next()?.unwrap().decode(DecodeHint::Primitive(PrimitiveType::U64))? {
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
        // println!("name={}", dir.path().to_str().unwrap());
        // if let Ok(contents) = std::str::from_utf8(&contents) {
        //     println!("{}", contents);
        // } else {
        //     println!("<err>");
        // }
        let output = parse_json::<JsonValue>(&contents, &mut Context::new());
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
