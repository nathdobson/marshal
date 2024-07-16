#![cfg(test)]
#![allow(soft_unstable)]
#![feature(test)]
#![deny(unused_must_use)]
extern crate test;

use marshal::context::OwnedContext;
use marshal::{Deserialize, Serialize};
use marshal_bin::decode::full::BinDecoderBuilder;
use marshal_bin::decode::BinDecoderSchema;
use marshal_bin::encode::full::BinEncoderBuilder;
use marshal_bin::encode::BinEncoderSchema;
use marshal_fixed::decode::full::FixedDecoderBuilder;
use marshal_fixed::encode::full::FixedEncoderBuilder;
use marshal_json::decode::full::JsonDecoderBuilder;
use marshal_json::encode::full::JsonEncoderBuilder;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::hint::black_box;
use std::iter;
use test::Bencher;

#[derive(Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd, Hash, Debug, Copy, Clone)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

fn create_value() -> Vec<Vec3> {
    let mut rng = XorShiftRng::seed_from_u64(12474343);
    iter::repeat_with(|| Vec3 {
        x: rng.gen_range(10i32..100000),
        y: rng.gen_range(10i32..100000),
        z: rng.gen_range(10i32..100000),
    })
    .take(10000)
    .collect()
}

#[bench]
fn bench_encode_json(b: &mut Bencher) {
    let value = black_box(create_value());
    b.iter(|| {
        black_box(
            JsonEncoderBuilder::new()
                .serialize(&value, OwnedContext::new().borrow())
                .unwrap(),
        );
    })
}

#[bench]
fn bench_decode_json(b: &mut Bencher) {
    let value = black_box(create_value());
    let encoded = black_box(
        JsonEncoderBuilder::new()
            .serialize(&value, OwnedContext::new().borrow())
            .unwrap()
            .into_bytes(),
    );
    b.iter(|| {
        let output = black_box(
            JsonDecoderBuilder::new(&encoded)
                .deserialize::<Vec<Vec3>>(OwnedContext::new().borrow())
                .unwrap(),
        );
        assert_eq!(output, value);
    })
}

#[bench]
fn bench_encode_bin(b: &mut Bencher) {
    let value = black_box(create_value());
    b.iter(|| {
        black_box(
            BinEncoderBuilder::new(&mut BinEncoderSchema::new())
                .serialize(&value, OwnedContext::new().borrow())
                .unwrap(),
        );
    })
}

#[bench]
fn bench_decode_bin(b: &mut Bencher) {
    let value = black_box(create_value());
    let encoded = black_box(
        BinEncoderBuilder::new(&mut BinEncoderSchema::new())
            .serialize(&value, OwnedContext::new().borrow())
            .unwrap(),
    );
    b.iter(|| {
        let output = black_box(
            BinDecoderBuilder::new(&encoded, &mut BinDecoderSchema::new())
                .deserialize::<Vec<Vec3>>(OwnedContext::new().borrow())
                .unwrap(),
        );
        assert_eq!(output, value);
    })
}

#[bench]
fn bench_encode_flat(b: &mut Bencher) {
    let value = black_box(create_value());
    b.iter(|| {
        black_box(
            FixedEncoderBuilder::new()
                .serialize(&value, OwnedContext::new().borrow())
                .unwrap(),
        );
    })
}

#[bench]
fn bench_decode_flat(b: &mut Bencher) {
    let value = black_box(create_value());
    let encoded = black_box(
        FixedEncoderBuilder::new()
            .serialize(&value, OwnedContext::new().borrow())
            .unwrap(),
    );
    b.iter(|| {
        let output = black_box(
            FixedDecoderBuilder::new(&encoded)
                .deserialize::<Vec<Vec3>>(OwnedContext::new().borrow())
                .unwrap(),
        );
        assert_eq!(output, value);
    })
}
