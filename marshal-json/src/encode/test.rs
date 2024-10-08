use marshal_core::encode::AnySpecEncoder;
use marshal_core::Primitive;

use crate::encode::SimpleJsonSpecEncoder;

#[track_caller]
fn run_simple(
    expected: &str,
    f: impl FnOnce(AnySpecEncoder<SimpleJsonSpecEncoder>) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let mut w = SimpleJsonSpecEncoder::new();
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
    let mut w = SimpleJsonSpecEncoder::new();
    w.start().encode_str("")?;
    assert_eq!(w.end()?, "\"\"");
    Ok(())
}

#[test]
fn test_ascii() -> anyhow::Result<()> {
    let mut w = SimpleJsonSpecEncoder::new();
    w.start().encode_str("abc")?;
    assert_eq!(w.end()?, "\"abc\"");
    Ok(())
}

#[test]
fn test_escape() -> anyhow::Result<()> {
    let mut w = SimpleJsonSpecEncoder::new();
    w.start()
        .encode_str("\" \\ \n \r \u{0000} ' \u{000b} \t \u{000c} \u{0008}")?;
    assert_eq!(w.end()?, r#""\" \\ \n \r \u0000 ' \u000b \t \f \b""#);
    Ok(())
}

#[test]
fn test_surrogate() -> anyhow::Result<()> {
    let mut w = SimpleJsonSpecEncoder::new();
    w.start().encode_str("🫎")?;
    assert_eq!(w.end()?, r#""🫎""#);
    Ok(())
}

#[test]
fn test_map0() -> anyhow::Result<()> {
    run_simple(r#"{}"#, |w| {
        let mut m = w.encode_map(0)?;
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
            let mut m = w.encode_map(1)?;
            let mut e = m.encode_entry()?;
            e.encode_key()?.encode_str("key")?;
            e.encode_value()?.encode_str("value")?;
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
            let mut m = w.encode_map(2)?;
            {
                let mut e = m.encode_entry()?;
                e.encode_key()?.encode_str("k1")?;
                e.encode_value()?.encode_str("v1")?;
                e.end()?;
            }
            {
                let mut e = m.encode_entry()?;
                e.encode_key()?.encode_str("k2")?;
                e.encode_value()?.encode_str("v2")?;
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
        let mut s = w.encode_seq(0)?;
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
            let mut s = w.encode_seq(1)?;
            s.encode_element()?.encode_str("elem")?;
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
            let mut s = w.encode_seq(2)?;
            s.encode_element()?.encode_str("elem1")?;
            s.encode_element()?.encode_str("elem2")?;
            s.end()?;
            Ok(())
        },
    )
}

#[test]
fn test_none() -> anyhow::Result<()> {
    run_simple(r#"null"#, |w| {
        w.encode_none()?;
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
            let mut w = w.encode_some()?;
            w.encode_some()?.encode_none()?;
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
            let mut w = w.encode_some()?;
            let mut w2 = w.encode_some()?.encode_some()?;
            w2.encode_some()?.encode_none()?;
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
            let mut w = w.encode_some()?;
            let mut w2 = w.encode_some()?.encode_some()?;
            let mut w3 = w2.encode_some()?.encode_some()?;
            w3.encode_some()?.encode_none()?;
            w3.end()?;
            w2.end()?;
            w.end()?;
            Ok(())
        },
    )
}

#[test]
fn test_prim() -> anyhow::Result<()> {
    run_simple("false", |w| w.encode_prim(Primitive::Bool(false)))?;
    run_simple("true", |w| w.encode_prim(Primitive::Bool(true)))?;
    run_simple("null", |w| w.encode_prim(Primitive::Unit))?;
    run_simple("123", |w| w.encode_prim(Primitive::I8(123)))?;
    run_simple("123.1", |w| w.encode_prim(Primitive::F32(123.1)))?;
    Ok(())
}

#[test]
fn test_tuple() -> anyhow::Result<()> {
    run_simple("[]", |w| {
        let mut w = w.encode_tuple(0)?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4\n]", |w| {
        let mut w = w.encode_tuple(0)?;
        w.encode_element()?.encode_prim(Primitive::U32(4))?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4,\n  8\n]", |w| {
        let mut w = w.encode_tuple(0)?;
        w.encode_element()?.encode_prim(Primitive::U32(4))?;
        w.encode_element()?.encode_prim(Primitive::U32(8))?;
        w.end()?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn test_seq() -> anyhow::Result<()> {
    run_simple("[]", |w| {
        let mut w = w.encode_seq(0)?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4\n]", |w| {
        let mut w = w.encode_seq(1)?;
        w.encode_element()?.encode_prim(Primitive::U32(4))?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4,\n  8\n]", |w| {
        let mut w = w.encode_seq(2)?;
        w.encode_element()?.encode_prim(Primitive::U32(4))?;
        w.encode_element()?.encode_prim(Primitive::U32(8))?;
        w.end()?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn test_tuple_struct() -> anyhow::Result<()> {
    run_simple("[]", |w| {
        let mut w = w.encode_tuple_struct("", 0)?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4\n]", |w| {
        let mut w = w.encode_tuple_struct("", 0)?;
        w.encode_field()?.encode_prim(Primitive::U32(4))?;
        w.end()?;
        Ok(())
    })?;
    run_simple("[\n  4,\n  8\n]", |w| {
        let mut w = w.encode_tuple_struct("", 0)?;
        w.encode_field()?.encode_prim(Primitive::U32(4))?;
        w.encode_field()?.encode_prim(Primitive::U32(8))?;
        w.end()?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn test_struct() -> anyhow::Result<()> {
    run_simple("{}", |w| {
        let mut w = w.encode_struct("", &[])?;
        w.end()?;
        Ok(())
    })?;
    run_simple(
        r#"
{
  "a": 4
}"#,
        |w| {
            let mut w = w.encode_struct("", &["a"])?;
            w.encode_field()?.encode_prim(Primitive::U32(4))?;
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
            let mut w = w.encode_struct("", &["b", "c"])?;
            w.encode_field()?.encode_prim(Primitive::U32(4))?;
            w.encode_field()?.encode_prim(Primitive::U32(8))?;
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
            let mut w = w.encode_tuple_variant("", &["x", "y"], 0, 0)?;
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
            let mut w = w.encode_tuple_variant("", &["x", "y"], 1, 1)?;
            w.encode_field()?.encode_prim(Primitive::U32(4))?;
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
            let mut w = w.encode_tuple_variant("", &["x", "y", "z"], 2, 2)?;
            w.encode_field()?.encode_prim(Primitive::U32(4))?;
            w.encode_field()?.encode_prim(Primitive::U32(8))?;
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
            let mut w = w.encode_struct_variant("", &["x", "y"], 0, &[])?;
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
            let mut w = w.encode_struct_variant("", &["x", "y"], 0, &["a"])?;
            w.encode_field()?.encode_prim(Primitive::U32(4))?;
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
            let mut w = w.encode_struct_variant("", &["x", "y"], 0, &["b", "c"])?;
            w.encode_field()?.encode_prim(Primitive::U32(4))?;
            w.encode_field()?.encode_prim(Primitive::U32(8))?;
            w.end()?;
            Ok(())
        },
    )?;
    Ok(())
}
