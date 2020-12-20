use crate::Point;
use crate::Progress;
use crate::Status;

#[derive(Debug)]
pub struct SlicePoint<'a, T: 'a> {
    pub offset: usize,
    pub s: &'a [T],
}

impl<'a, T: 'a> SlicePoint<'a, T> {
    #[inline]
    pub fn new(slice: &'a [T]) -> Self {
        SlicePoint {
            offset: 0,
            s: slice,
        }
    }

    #[inline]
    pub fn advance_by(self, offset: usize) -> Self {
        SlicePoint {
            s: &self.s[offset..],
            offset: self.offset + offset,
        }
    }

    #[inline]
    pub fn fail<U>(self) -> Progress<SlicePoint<'a, T>, U, ()> {
        Progress {
            point: self,
            status: Status::Failure(()),
        }
    }

    #[inline]
    pub fn success<E>(self, len: usize) -> Progress<SlicePoint<'a, T>, &'a [T], E> {
        let matched = &self.s[..len];
        let rest = &self.s[len..];

        Progress {
            point: SlicePoint {
                s: rest,
                offset: self.offset + len,
            },
            status: Status::Success(matched),
        }
    }

    #[inline]
    pub fn success_opt(self, len: Option<usize>) -> Progress<SlicePoint<'a, T>, &'a [T], ()> {
        if let Some(l) = len {
            self.success(l)
        } else {
            self.fail()
        }
    }

    #[inline]
    pub fn consume_opt(self, len: Option<usize>) -> Progress<SlicePoint<'a, T>, &'a [T], ()> {
        if let Some(l) = len {
            self.consume(l)
        } else {
            self.fail()
        }
    }

    #[inline]
    pub fn consume(self, len: usize) -> Progress<SlicePoint<'a, T>, &'a [T], ()> {
        if len > 0 && len <= self.s.len() {
            self.success(len)
        } else {
            self.fail()
        }
    }
}

impl<'a, T> Point for SlicePoint<'a, T> {
    #[inline]
    fn zero() -> Self {
        SlicePoint { offset: 0, s: &[] }
    }
}

impl<'a, T> Copy for SlicePoint<'a, T> {}
impl<'a, T> Clone for SlicePoint<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> PartialOrd for SlicePoint<'a, T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Ord for SlicePoint<'a, T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl<'a, T> PartialEq for SlicePoint<'a, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.offset.eq(&other.offset)
    }
}

impl<'a, T> Eq for SlicePoint<'a, T> {}

impl<'a, T: PartialEq> SlicePoint<'a, T> {
    #[inline]
    pub fn tag(
        tag: &'a [T],
    ) -> impl Fn(SlicePoint<'a, T>) -> Progress<SlicePoint<'a, T>, &'a [T], ()> {
        move |p| {
            p.success_opt(
                p.s.get(0..tag.len())
                    .filter(|bytes| bytes == &tag)
                    .map(|b| b.len()),
            )
        }
    }
}
