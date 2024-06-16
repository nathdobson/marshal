#![deny(unused_must_use)]

use criterion::{black_box, Criterion, criterion_group, criterion_main};
use rand::{RngCore, SeedableRng};
use rand::seq::SliceRandom;
use rand_xorshift::XorShiftRng;
use serde::{Deserialize, Serialize};

fn bincode_mono<T: Serialize + for<'de> Deserialize<'de>>(value: &T) {
    black_box(bincode::deserialize::<T>(&black_box(bincode::serialize(&value)).unwrap()).unwrap());
}
fn bincode_dyn<T: Serialize + for<'de> Deserialize<'de>>(value: &T) {
    black_box(bincode::deserialize::<T>(&black_box(bincode::serialize(&value)).unwrap()).unwrap());
}

fn json_mono<T: Serialize + for<'de> Deserialize<'de>>(value: &T) {
    black_box(
        serde_json::from_slice::<T>(&black_box(serde_json::to_vec(&value)).unwrap()).unwrap(),
    );
}

fn json_dyn<T: Serialize + for<'de> Deserialize<'de>>(value: &T) {
    black_box(
        serde_json::from_slice::<T>(&black_box(serde_json::to_vec(&value)).unwrap()).unwrap(),
    );
}

#[derive(Serialize, Deserialize, Copy, Clone)]
enum Value {
    A(u32),
    B(u64),
}

#[typetag::serde(tag = "type")]
trait Foo {}

#[typetag::serde]
impl Foo for Vec<Value> {}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = XorShiftRng::seed_from_u64(105555);
    let vec: Box<Vec<Value>> = Box::new(
        (0..1024)
            .map(|_| {
                *[Value::A(rng.next_u32()), Value::B(rng.next_u64())]
                    .choose(&mut rng)
                    .unwrap()
            })
            .collect::<Vec<_>>(),
    );
    let boxed = vec.clone() as Box<dyn Foo>;
    c.bench_function("bincode mono", |b| {
        b.iter(|| bincode_mono::<Box<Vec<Value>>>(black_box(&vec)))
    });
    c.bench_function("bincode dyn", |b| {
        b.iter(|| bincode_dyn::<Box<dyn Foo>>(black_box(&boxed)))
    });
    c.bench_function("json mono", |b| {
        b.iter(|| json_mono::<Box<Vec<Value>>>(black_box(&vec)))
    });
    c.bench_function("json dyn", |b| {
        b.iter(|| json_dyn::<Box<dyn Foo>>(black_box(&boxed)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
