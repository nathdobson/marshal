use crate::text::any::{TextAny, TextAnyParser, TextAnyPosition};
use crate::text::depth_budget::DepthBudget;
use crate::text::error::TextResult;
use crate::text::value::into_json_value;
use crate::text::TextParser;
use std::fs;
use std::fs::read_dir;

#[test]
fn test() -> TextResult<()> {
    let input = b"[1,23]";
    let mut p = TextParser::new(input);
    let mut p = TextAnyParser::new(&mut p, TextAnyPosition::Any, DepthBudget::new(1));
    match p.parse_any()? {
        TextAny::TextSeqParser(mut p) => {
            match p.next()?.unwrap().parse_any()? {
                TextAny::Number(mut n) => assert_eq!(1, n.parse_number::<u32>()?),
                _ => todo!(),
            }
            match p.next()?.unwrap().parse_any()? {
                TextAny::Number(mut n) => assert_eq!(23, n.parse_number::<u32>()?),
                _ => todo!(),
            }
            assert!(p.next()?.is_none());
            p.end()?;
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
        let mut parser = TextParser::new(&contents);
        println!("name={}", dir.path().to_str().unwrap());
        if let Ok(contents) = std::str::from_utf8(&contents) {
            println!("{}", contents);
        } else {
            println!("<err>");
        }
        let output = into_json_value(&mut parser);
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
