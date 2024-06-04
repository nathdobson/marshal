use serde_json::Value;
use std::fs;
use std::fs::read_dir;

use crate::context::DeserializeContext;
use crate::error::ParseError;
use crate::json::value::parse_json;
use crate::json::{JsonParser, SingletonContext};
use crate::simple::SimpleAnyParser;
use crate::{AnyParser, ParseHint, ParserView, SeqParser};

#[test]
fn test() -> Result<(), ParseError> {
    let input = b"[1,23]";
    let mut p = JsonParser::new(input);
    let p = SimpleAnyParser::new(&mut p, SingletonContext::default());
    match p.parse(ParseHint::Any)? {
        ParserView::Seq(mut p) => {
            match p.parse_next()?.unwrap().parse(ParseHint::U64)? {
                ParserView::U64(x) => assert_eq!(x, 1),
                _ => todo!(),
            }
            match p.parse_next()?.unwrap().parse(ParseHint::U64)? {
                ParserView::U64(x) => assert_eq!(x, 23),
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
