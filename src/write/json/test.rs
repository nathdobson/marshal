use crate::write::json::JsonWriter;
use crate::write::{AnyWriter, EntryWriter, MapWriter};

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
fn test_map() -> anyhow::Result<()> {
    let mut w = JsonWriter::new();
    let mut m = w.start().write_map(None)?;
    let mut e = m.write_entry()?;
    e.write_key()?.write_str("key")?;
    e.write_value()?.write_str("value")?;
    e.end()?;
    m.end()?;
    assert_eq!(
        format!("\n{}", w.end()?),
        r#"
{
  "key": "value"
}"#
    );
    Ok(())
}
