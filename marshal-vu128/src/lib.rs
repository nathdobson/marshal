#![feature(test)]
#![feature(slice_take)]

#[cfg(test)]
mod test;

use std::array::TryFromSliceError;
use std::borrow::Borrow;
use vu128::{
    decode_f32, decode_f64, decode_i128, decode_i32, decode_i64, decode_u128, decode_u32,
    decode_u64, encode_f32, encode_f64, encode_i128, encode_i32, encode_i64, encode_u128,
    encode_u32, encode_u64,
};

pub trait Array: Default + Borrow<[Self::Item]> {
    type Item;
    const ARRAY_LEN: usize;
    fn try_from_slice(slice: &[Self::Item]) -> Result<&Self, TryFromSliceError>;
    fn try_from_slice_mut(slice: &mut [Self::Item]) -> Result<&mut Self, TryFromSliceError>;
}

impl<const N: usize, T> Array for [T; N]
where
    [T; N]: Default,
{
    type Item = T;
    const ARRAY_LEN: usize = N;
    #[inline]
    fn try_from_slice(slice: &[Self::Item]) -> Result<&Self, TryFromSliceError> {
        slice.try_into()
    }
    #[inline]
    fn try_from_slice_mut(slice: &mut [Self::Item]) -> Result<&mut Self, TryFromSliceError> {
        slice.try_into()
    }
}

pub trait ToFromVu128: Sized {
    type Buffer: Array<Item = u8>;
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize;
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize);
}

impl ToFromVu128 for bool {
    type Buffer = [u8; 1];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        buf[0] = value as u8;
        1
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        (buf[0] != 0, 1)
    }
}

impl ToFromVu128 for u8 {
    type Buffer = [u8; 5];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_u32(buf, value as u32)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        let (v, c) = decode_u32(buf);
        (v as u8, c)
    }
}

impl ToFromVu128 for i8 {
    type Buffer = [u8; 5];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_i32(buf, value as i32)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        let (v, c) = decode_i32(buf);
        (v as i8, c)
    }
}

impl ToFromVu128 for u16 {
    type Buffer = [u8; 5];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_u32(buf, value as u32)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        let (v, c) = decode_u32(buf);
        (v as u16, c)
    }
}

impl ToFromVu128 for i16 {
    type Buffer = [u8; 5];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_i32(buf, value as i32)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        let (v, c) = decode_i32(buf);
        (v as i16, c)
    }
}

impl ToFromVu128 for u32 {
    type Buffer = [u8; 5];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_u32(buf, value)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        decode_u32(buf)
    }
}

impl ToFromVu128 for i32 {
    type Buffer = [u8; 5];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_i32(buf, value)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        decode_i32(buf)
    }
}

impl ToFromVu128 for f32 {
    type Buffer = [u8; 5];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_f32(buf, value)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        decode_f32(buf)
    }
}

impl ToFromVu128 for u64 {
    type Buffer = [u8; 9];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_u64(buf, value)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        decode_u64(buf)
    }
}

impl ToFromVu128 for i64 {
    type Buffer = [u8; 9];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_i64(buf, value)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        decode_i64(buf)
    }
}

impl ToFromVu128 for f64 {
    type Buffer = [u8; 9];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_f64(buf, value)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        decode_f64(buf)
    }
}

impl ToFromVu128 for u128 {
    type Buffer = [u8; 17];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_u128(buf, value)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        decode_u128(buf)
    }
}

impl ToFromVu128 for i128 {
    type Buffer = [u8; 17];
    #[inline]
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_i128(buf, value)
    }
    #[inline]
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        decode_i128(buf)
    }
}

pub trait WriteVu128 {
    fn write_vu128<T: ToFromVu128>(&mut self, value: T);
}

impl WriteVu128 for Vec<u8> {
    #[inline]
    fn write_vu128<T: ToFromVu128>(&mut self, value: T) {
        let mut bytes = T::Buffer::default();
        let len = T::encode_vu128(&mut bytes, value);
        self.extend_from_slice(&bytes.borrow()[0..len]);
    }
}

pub trait ReadVu128 {
    fn read_vu128<T: ToFromVu128>(&mut self) -> Result<T, TryFromSliceError>;
}

impl<'a> ReadVu128 for &'a [u8] {
    #[inline]
    fn read_vu128<T: ToFromVu128>(&mut self) -> Result<T, TryFromSliceError> {
        let (value, len) =
            T::decode_vu128(T::Buffer::try_from_slice(&self[..T::Buffer::ARRAY_LEN])?);
        *self = &self[len..];
        Ok(value)
    }
}

pub const VU128_PADDING: usize = 17;
