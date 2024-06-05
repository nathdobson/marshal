use crate::write::json::JsonWriter;
use crate::write::simple::SimpleAnyWriter;
use crate::write::{
    AnyWriter, EntryWriter, MapWriter, SeqWriter, SomeWriter, StructVariantWriter, StructWriter,
    TupleStructWriter, TupleVariantWriter, TupleWriter,
};
use crate::Primitive;

#[track_caller]
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

#[test]
fn test_none() -> anyhow::Result<()> {
    run_simple(r#"null"#, |w| {
        w.write_none()?;
        Ok(())
    })
}

#[test]
fn test_some_none() -> anyhow::Result<()> {
    run_simple(
        r#"
{
  "None": null
}"#,
        |w| {
            let mut w = w.write_some()?;
            w.write_some()?.write_none()?;
            w.end()?;
            Ok(())
        },
    )
}
#[test]
fn test_some_some_none() -> anyhow::Result<()> {
    run_simple(
        r#"
{
  "Some": null
}"#,
        |w| {
            let mut w = w.write_some()?;
            let mut w2 = w.write_some()?.write_some()?;
            w2.write_some()?.write_none()?;
            w2.end()?;
            w.end()?;
            Ok(())
        },
    )
}

#[test]
fn test_some_some_some_none() -> anyhow::Result<()> {
    run_simple(
        r#"
{
  "Some": {
    "None": null
  }
}"#,
        |w| {
            let mut w = w.write_some()?;
            let mut w2 = w.write_some()?.write_some()?;
            let mut w3 = w2.write_some()?.write_some()?;
            w3.write_some()?.write_none()?;
            w3.end()?;
            w2.end()?;
            w.end()?;
            Ok(())
        },
    )
}

#[test]
fn test_prim() -> anyhow::Result<()> {
    run_simple("false", |w| w.write_prim(Primitive::Bool(false)))?;
    run_simple("true", |w| w.write_prim(Primitive::Bool(true)))?;
    run_simple("null", |w| w.write_prim(Primitive::Unit))?;
    run_simple("123", |w| w.write_prim(Primitive::I8(123)))?;
    run_simple("123.1", |w| w.write_prim(Primitive::F32(123.1)))?;
    Ok(())
}

#[test]
fn test_tuple() -> anyhow::Result<()> {
    run_simple("[]", |w| {
        let mut w = w.write_tuple(0)?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4\n]", |w| {
        let mut w = w.write_tuple(0)?;
        w.write_element()?.write_prim(Primitive::U32(4))?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4,\n  8\n]", |w| {
        let mut w = w.write_tuple(0)?;
        w.write_element()?.write_prim(Primitive::U32(4))?;
        w.write_element()?.write_prim(Primitive::U32(8))?;
        w.end()?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn test_seq() -> anyhow::Result<()> {
    run_simple("[]", |w| {
        let mut w = w.write_seq(None)?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4\n]", |w| {
        let mut w = w.write_seq(None)?;
        w.write_element()?.write_prim(Primitive::U32(4))?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4,\n  8\n]", |w| {
        let mut w = w.write_seq(None)?;
        w.write_element()?.write_prim(Primitive::U32(4))?;
        w.write_element()?.write_prim(Primitive::U32(8))?;
        w.end()?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn test_tuple_struct() -> anyhow::Result<()> {
    run_simple("[]", |w| {
        let mut w = w.write_tuple_struct("", 0)?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4\n]", |w| {
        let mut w = w.write_tuple_struct("", 0)?;
        w.write_field()?.write_prim(Primitive::U32(4))?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4,\n  8\n]", |w| {
        let mut w = w.write_tuple_struct("", 0)?;
        w.write_field()?.write_prim(Primitive::U32(4))?;
        w.write_field()?.write_prim(Primitive::U32(8))?;
        w.end()?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn test_struct() -> anyhow::Result<()> {
    run_simple("{}", |w| {
        let mut w = w.write_struct("", 0)?;
        w.end()?;
        Ok(())
    })?;
    run_simple(
        r#"
{
  "a": 4
}"#,
        |w| {
            let mut w = w.write_struct("", 0)?;
            w.write_field("a")?.write_prim(Primitive::U32(4))?;
            w.end()?;
            Ok(())
        },
    )?;
    run_simple(
        r#"
{
  "b": 4,
  "c": 8
}"#,
        |w| {
            let mut w = w.write_struct("", 0)?;
            w.write_field("b")?.write_prim(Primitive::U32(4))?;
            w.write_field("c")?.write_prim(Primitive::U32(8))?;
            w.end()?;
            Ok(())
        },
    )?;
    Ok(())
}

#[test]
fn test_tuple_variant() -> anyhow::Result<()> {
    run_simple(
        r#"
{
  "x": []
}"#,
        |w| {
            let mut w = w.write_tuple_variant("", 0, "x", 0)?;
            w.end()?;
            Ok(())
        },
    )?;
    run_simple(
        r#"
{
  "y": [
    4
  ]
}"#,
        |w| {
            let mut w = w.write_tuple_variant("", 0, "y", 1)?;
            w.write_field()?.write_prim(Primitive::U32(4))?;
            w.end()?;
            Ok(())
        },
    )?;
    run_simple(
        r#"
{
  "z": [
    4,
    8
  ]
}"#,
        |w| {
            let mut w = w.write_tuple_variant("", 0, "z", 2)?;
            w.write_field()?.write_prim(Primitive::U32(4))?;
            w.write_field()?.write_prim(Primitive::U32(8))?;
            w.end()?;
            Ok(())
        },
    )?;
    Ok(())
}

#[test]
fn test_struct_variant() -> anyhow::Result<()> {
    run_simple(
        r#"
{
  "x": {}
}"#,
        |w| {
            let mut w = w.write_struct_variant("", 0, "x", 0)?;
            w.end()?;
            Ok(())
        },
    )?;
    run_simple(
        r#"
{
  "x": {
    "a": 4
  }
}"#,
        |w| {
            let mut w = w.write_struct_variant("", 0, "x", 1)?;
            w.write_field("a")?.write_prim(Primitive::U32(4))?;
            w.end()?;
            Ok(())
        },
    )?;
    run_simple(
        r#"
{
  "x": {
    "b": 4,
    "c": 8
  }
}"#,
        |w| {
            let mut w = w.write_struct_variant("", 0, "x", 2)?;
            w.write_field("b")?.write_prim(Primitive::U32(4))?;
            w.write_field("c")?.write_prim(Primitive::U32(8))?;
            w.end()?;
            Ok(())
        },
    )?;
    Ok(())
}
