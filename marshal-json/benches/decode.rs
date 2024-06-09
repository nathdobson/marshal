use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use marshal::context::Context;
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::value::JsonValue;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::Duration;

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
    // g.warm_up_time(Duration::from_nanos(1));
    // g.measurement_time(Duration::from_secs(1));
    // let data = fs::read("nativejson-benchmark/data/canada.json").unwrap();
    // let data = fs::read("nativejson-benchmark/data/citm_catalog.json").unwrap();
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
