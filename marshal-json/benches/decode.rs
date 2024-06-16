use std::fs;
use std::path::Path;

use criterion::{BenchmarkId, black_box, Criterion, criterion_group, criterion_main};
use serde_json::Value;

use marshal::context::Context;
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::value::JsonValue;

fn parse_serde(data: &[u8]) {
    black_box(serde_json::from_slice::<Value>(black_box(data)).unwrap());
}

fn parse_marshal(data: &[u8]) {
    black_box(
        JsonDecoderBuilder::new(black_box(data))
            .deserialize::<JsonValue>(&mut Context::new())
            .unwrap(),
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("parse");
    for name in &["canada.json", "citm_catalog.json", "twitter.json"] {
        let data = fs::read(Path::new("nativejson-benchmark/data").join(name)).unwrap();
        g.bench_function(BenchmarkId::new("parse_serde", name), |b| {
            b.iter(|| parse_serde(&data))
        });
        g.bench_function(BenchmarkId::new("parse_marshal", name), |b| {
            b.iter(|| parse_marshal(&data))
        });
    }
    g.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
