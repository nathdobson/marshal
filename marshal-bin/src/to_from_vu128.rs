use std::array::TryFromSliceError;
use vu128::{decode_u32, encode_u32};

pub trait Array {
    type Item;
    const ARRAY_LEN: usize;
    fn try_from_slice(slice: &[Self::Item]) -> Result<&Self, TryFromSliceError>;
    fn try_from_slice_mut(slice: &mut [Self::Item]) -> Result<&mut Self, TryFromSliceError>;
}

impl<const N: usize, T> Array for [T; N] {
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

impl ToFromVu128 for u32 {
    type Buffer = [u8; 5];
    fn encode_vu128(buf: &mut Self::Buffer, value: Self) -> usize {
        encode_u32(buf, value)
    }
    fn decode_vu128(buf: &Self::Buffer) -> (Self, usize) {
        decode_u32(buf)
    }
}
