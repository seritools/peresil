use crate::bytes::BytePoint;
use crate::Point;
use crate::Progress;

/// Trait defining simple byte parsers for numeric primitives
/// in both little-endian and big-endian encodings.
pub trait ParseNumber: Point {
    fn u8_le(self) -> Progress<Self, u8, ()>;
    fn u8_be(self) -> Progress<Self, u8, ()>;
    fn u16_le(self) -> Progress<Self, u16, ()>;
    fn u16_be(self) -> Progress<Self, u16, ()>;
    fn u32_le(self) -> Progress<Self, u32, ()>;
    fn u32_be(self) -> Progress<Self, u32, ()>;
    fn u64_le(self) -> Progress<Self, u64, ()>;
    fn u64_be(self) -> Progress<Self, u64, ()>;
    fn u128_le(self) -> Progress<Self, u128, ()>;
    fn u128_be(self) -> Progress<Self, u128, ()>;

    fn i8_le(self) -> Progress<Self, i8, ()>;
    fn i8_be(self) -> Progress<Self, i8, ()>;
    fn i16_le(self) -> Progress<Self, i16, ()>;
    fn i16_be(self) -> Progress<Self, i16, ()>;
    fn i32_le(self) -> Progress<Self, i32, ()>;
    fn i32_be(self) -> Progress<Self, i32, ()>;
    fn i64_le(self) -> Progress<Self, i64, ()>;
    fn i64_be(self) -> Progress<Self, i64, ()>;
    fn i128_le(self) -> Progress<Self, i128, ()>;
    fn i128_be(self) -> Progress<Self, i128, ()>;

    fn f32_le(self) -> Progress<Self, f32, ()>;
    fn f32_be(self) -> Progress<Self, f32, ()>;
    fn f64_le(self) -> Progress<Self, f64, ()>;
    fn f64_be(self) -> Progress<Self, f64, ()>;
}

macro_rules! impl_number {
    ($num:ident) => {
        paste::paste! {
            #[doc = "Parses a `" $num "` in little-endian encoding."]
            #[inline]
            fn [<$num _le>](self) -> Progress<Self, $num, ()> {
                self
                    .consume(::std::mem::size_of::<$num>())
                    .map(|n| {
                        // unwrap cannot fail since n.len() is always at least as big
                        // as the number type, because `consume` consumed at least
                        // that many bytes if we end up here
                        $num::from_le_bytes(::std::convert::TryInto::try_into(n).unwrap())
                    })
            }

            #[doc = "Parses a `" $num "` in big-endian encoding."]
            #[inline]
            fn [<$num _be>](self) -> Progress<Self, $num, ()> {
                self
                    .consume(::std::mem::size_of::<$num>())
                    .map(|n| {
                        // unwrap cannot fail since n.len() is always at least as big
                        // as the number type, because `consume` consumed at least
                        // that many bytes if we end up here
                        $num::from_be_bytes(::std::convert::TryInto::try_into(n).unwrap())
                    })
            }
        }
    };

    ($ty:ident $($tys:ident)*) => {
        impl_number!($ty);
        impl_number!($($tys)*);
    };
}

impl ParseNumber for BytePoint<'_> {
    impl_number!(
        u8 u16 u32 u64 u128
        i8 i16 i32 i64 i128
        f32 f64
    );
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Status;

    #[test]
    fn fails_with_too_short_input() {
        let p = BytePoint { offset: 0, s: &[] };

        let expected_u64 = Progress {
            point: p,
            status: Status::Failure(()),
        };
        let expected_i8 = Progress {
            point: p,
            status: Status::Failure(()),
        };

        assert_eq!(p.u64_le(), expected_u64);
        assert_eq!(p.u64_be(), expected_u64);
        assert_eq!(p.i8_le(), expected_i8);
        assert_eq!(p.i8_be(), expected_i8);
    }

    #[test]
    fn parses_ints_correctly() {
        let p = BytePoint {
            offset: 0,
            s: &[0x01, 0x02, 0x03, 0x04, 0xD0, 0x0D, 0xF0, 0x0D],
        };

        assert_eq!(
            p.u64_le(),
            Progress {
                point: BytePoint { offset: 8, s: &[] },
                status: Status::Success(0x0D_F0_0D_D0_04_03_02_01_u64),
            }
        );
        assert_eq!(
            p.i16_le(),
            Progress {
                point: BytePoint {
                    offset: 2,
                    s: &[0x03, 0x04, 0xD0, 0x0D, 0xF0, 0x0D]
                },
                status: Status::Success(0x02_01_i16),
            }
        );

        assert_eq!(
            p.u64_be(),
            Progress {
                point: BytePoint { offset: 8, s: &[] },
                status: Status::Success(0x01_02_03_04_D0_0D_F0_0D_u64),
            }
        );
        assert_eq!(
            p.i16_be(),
            Progress {
                point: BytePoint {
                    offset: 2,
                    s: &[0x03, 0x04, 0xD0, 0x0D, 0xF0, 0x0D]
                },
                status: Status::Success(0x01_02_i16),
            }
        );
    }
}
