use std::fs;
use std::fs::read_dir;

use serde_json::Value;

use crate::{Primitive, PrimitiveType};
use crate::de::context::DeserializeContext;
use crate::parse::{AnyParser, ParseHint, ParserView, SeqParser};
use crate::parse::json::full::parse_json;
use crate::parse::json::SimpleJsonParser;
use crate::parse::simple::SimpleAnyParser;

#[test]
fn test() -> anyhow::Result<()> {
    let input = b"[1,23]";
    let mut p = SimpleJsonParser::new(input);
    let p = SimpleAnyParser::new(&mut p, crate::parse::json::AnyParser::default());
    match p.parse(ParseHint::Any)? {
        ParserView::Seq(mut p) => {
            match p.parse_next()?.unwrap().parse(ParseHint::Primitive(PrimitiveType::U64))? {
                ParserView::Primitive(Primitive::U64(x)) => assert_eq!(x, 1),
                _ => todo!(),
            }
            match p.parse_next()?.unwrap().parse(ParseHint::Primitive(PrimitiveType::U64))? {
                ParserView::Primitive(Primitive::U64(x)) => assert_eq!(x, 23),
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
    Ok(())
}

#[test]
fn test_parsing() {
    for dir in read_dir("JSONTestSuite/test_parsing").unwrap() {
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
        println!("name={}", dir.path().to_str().unwrap());
        if let Ok(contents) = std::str::from_utf8(&contents) {
            println!("{}", contents);
        } else {
            println!("<err>");
        }
        let output = parse_json::<Value>(&contents, &DeserializeContext::new());
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
