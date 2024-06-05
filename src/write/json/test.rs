use crate::write::json::{JsonAnyWriter, JsonWriter};
use crate::write::simple::SimpleAnyWriter;
use crate::write::{AnyWriter, EntryWriter, MapWriter, SeqWriter};

fn run_simple(
    expected: &str,
    f: impl FnOnce(SimpleAnyWriter<JsonWriter>) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let mut w = JsonWriter::new();
    f(w.start())?;
    let actual = w.end()?;
    let expected = expected.trim_start();
    if expected != actual {
        eprintln!("expected:\n{}", expected);
        eprintln!("actual:\n{}", actual);
        panic!();
    }
    Ok(())
}

#[test]
fn test_empty_string() -> anyhow::Result<()> {
    let mut w = JsonWriter::new();
    w.start().write_str("")?;
    assert_eq!(w.end()?, "\"\"");
    Ok(())
}

#[test]
fn test_ascii() -> anyhow::Result<()> {
    let mut w = JsonWriter::new();
    w.start().write_str("abc")?;
    assert_eq!(w.end()?, "\"abc\"");
    Ok(())
}

#[test]
fn test_escape() -> anyhow::Result<()> {
    let mut w = JsonWriter::new();
    w.start()
        .write_str("\" \\ \n \r \u{0000} ' \u{000b} \t \u{000c} \u{0008}")?;
    assert_eq!(w.end()?, r#""\" \\ \n \r \u0000 ' \u000b \t \f \b""#);
    Ok(())
}

#[test]
fn test_surrogate() -> anyhow::Result<()> {
    let mut w = JsonWriter::new();
    w.start().write_str("🫎")?;
    assert_eq!(w.end()?, r#""🫎""#);
    Ok(())
}

#[test]
fn test_map0() -> anyhow::Result<()> {
    run_simple(r#"{}"#, |w| {
        let mut m = w.write_map(None)?;
        m.end()?;
        Ok(())
    })
}

#[test]
fn test_map1() -> anyhow::Result<()> {
    run_simple(
        r#"
{
  "key": "value"
}"#,
        |w| {
            let mut m = w.write_map(None)?;
            let mut e = m.write_entry()?;
            e.write_key()?.write_str("key")?;
            e.write_value()?.write_str("value")?;
            e.end()?;
            m.end()?;
            Ok(())
        },
    )
}

#[test]
fn test_map2() -> anyhow::Result<()> {
    run_simple(
        r#"
{
  "k1": "v1",
  "k2": "v2"
}"#,
        |w| {
            let mut m = w.write_map(None)?;
            {
                let mut e = m.write_entry()?;
                e.write_key()?.write_str("k1")?;
                e.write_value()?.write_str("v1")?;
                e.end()?;
            }
            {
                let mut e = m.write_entry()?;
                e.write_key()?.write_str("k2")?;
                e.write_value()?.write_str("v2")?;
                e.end()?;
            }
            m.end()?;
            Ok(())
        },
    )
}

#[test]
fn test_seq0() -> anyhow::Result<()> {
    run_simple(r#"[]"#, |w| {
        let mut s = w.write_seq(None)?;
        s.end()?;
        Ok(())
    })
}

#[test]
fn test_seq1() -> anyhow::Result<()> {
    run_simple(
        r#"
[
  "elem"
]"#,
        |w| {
            let mut s = w.write_seq(None)?;
            s.write_element()?.write_str("elem")?;
            s.end()?;
            Ok(())
        },
    )
}

#[test]
fn test_seq2() -> anyhow::Result<()> {
    run_simple(
        r#"
[
  "elem1",
  "elem2"
]"#,
        |w| {
            let mut s = w.write_seq(None)?;
            s.write_element()?.write_str("elem1")?;
            s.write_element()?.write_str("elem2")?;
            s.end()?;
            Ok(())
        },
    )
}
