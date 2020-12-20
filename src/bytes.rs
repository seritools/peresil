use crate::SlicePoint;

mod num;
pub use self::num::ParseNumber;

pub type BytePoint<'a> = SlicePoint<'a, u8>;
