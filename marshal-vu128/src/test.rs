extern crate test;

use crate::{ReadVu128, WriteVu128, VU128_PADDING};
use rand::{thread_rng, Rng};
use std::hint::black_box;
use test::Bencher;
use vu128::{decode_u64, encode_u64};

const COUNT: usize = 10000;

fn create_original() -> Vec<u8> {
    let mut rng = thread_rng();
    let mut original = vec![];
    for _ in 0..COUNT {
        let n = 2.0f32.powf(rng.gen_range(0.0f32..30.0f32)) as u64;
        original.write_vu128(n);
    }
    original.resize(original.len() + VU128_PADDING, 0);
    original
}

#[bench]
fn bench_1(bencher: &mut Bencher) {
    let original = create_original();
    let mut output = vec![];
    bencher.iter(|| {
        output.clear();
        let mut input = original.as_slice();
        for _ in 0..black_box(COUNT) {
            let val: u64 = input.read_vu128().unwrap();
            output.write_vu128(val);
        }
    })
}

#[test]
fn test() {
    let n = 1722628047u64;
    let mut result = [255u8; 9];
    let len = encode_u64(&mut result, n);
    result[len..].fill(255);
    let (out, len2) = decode_u64(&result);
    println!("{:?} {:?}", len, result);
    println!("{:?} {:?}", out, len2);
}
