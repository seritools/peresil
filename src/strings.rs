use std::cmp::Ordering;

use crate::Point;
use crate::Progress;

/// Matches a literal string to a specific type, usually an enum.
pub type Identifier<'a, T> = (&'a str, T);

/// Tracks the location of parsing in a string, the most common case.
///
/// Helper methods are provided to do basic parsing tasks, such as
/// finding literal strings.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StringPoint<'a> {
    /// The portion of the input string to start parsing next
    pub s: &'a str,
    /// How far into the original string we are
    pub offset: usize,
}

impl<'a> PartialOrd for StringPoint<'a> {
    #[inline]
    fn partial_cmp(&self, other: &StringPoint<'a>) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<'a> Ord for StringPoint<'a> {
    #[inline]
    fn cmp(&self, other: &StringPoint<'a>) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl<'a> Point for StringPoint<'a> {
    fn zero() -> StringPoint<'a> {
        StringPoint { s: "", offset: 0 }
    }
}

impl<'a> StringPoint<'a> {
    #[inline]
    pub fn new(s: &'a str) -> StringPoint<'a> {
        StringPoint { s, offset: 0 }
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.s.is_empty()
    }

    /// Slices the string.
    #[inline]
    pub fn to(self, other: StringPoint<'a>) -> &'a str {
        let len = other.offset - self.offset;
        &self.s[..len]
    }

    #[inline]
    pub fn success<E>(self, len: usize) -> Progress<StringPoint<'a>, &'a str, E> {
        let matched = &self.s[..len];
        let rest = &self.s[len..];

        Progress::success(
            StringPoint {
                s: rest,
                offset: self.offset + len,
            },
            matched,
        )
    }

    #[inline]
    pub fn fail<T>(self) -> Progress<StringPoint<'a>, T, ()> {
        Progress::failure(self, ())
    }

    /// Advances the point by the number of bytes. If the value is
    /// `None`, then no value was able to be consumed, and the result
    /// is a failure.
    #[inline]
    pub fn consume_to(self, l: Option<usize>) -> Progress<StringPoint<'a>, &'a str, ()> {
        match l {
            None => self.fail(),
            Some(len) => self.success(len),
        }
    }

    /// Advances the point if it starts with the literal.
    #[inline]
    pub fn consume_literal(self, val: &str) -> Progress<StringPoint<'a>, &'a str, ()> {
        if self.s.starts_with(val) {
            self.success(val.len())
        } else {
            self.fail()
        }
    }

    /// Iterates through the identifiers and advances the point on the
    /// first matching identifier.
    #[inline]
    pub fn consume_identifier<T>(
        self,
        identifiers: &[Identifier<T>],
    ) -> Progress<StringPoint<'a>, T, ()>
    where
        T: Clone,
    {
        for (identifier, item) in identifiers {
            if self.s.starts_with(identifier) {
                return self
                    .consume_to(Some(identifier.len()))
                    .map(|_| item.clone())
                    .map_err(|_| unreachable!());
            }
        }

        self.fail()
    }
}

#[cfg(test)]
mod tests {
    use crate::{ParseMaster, Progress, Recoverable, Status, StringPoint};

    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct AnError(u8);

    impl Recoverable for AnError {
        fn recoverable(&self) -> bool {
            self.0 < 0x80
        }
    }

    type StringMaster<'a> = ParseMaster<StringPoint<'a>, AnError>;
    type StringProgress<'a, T> = Progress<StringPoint<'a>, T, AnError>;

    #[test]
    fn string_sequential() {
        fn all<'a>(pt: StringPoint<'a>) -> StringProgress<'a, (&'a str, &'a str, &'a str)> {
            let (pt, a) = try_parse!(pt.consume_literal("a").map_err(|_| AnError(1)));
            let (pt, b) = try_parse!(pt.consume_literal("b").map_err(|_| AnError(2)));
            let (pt, c) = try_parse!(pt.consume_literal("c").map_err(|_| AnError(3)));

            Progress {
                point: pt,
                status: Status::Success((a, b, c)),
            }
        }

        let mut d = ParseMaster::new();
        let pt = StringPoint::new("abc");

        let r = all(pt);
        let r = d.finish(r);

        assert_eq!(
            r,
            Progress {
                point: StringPoint { s: "", offset: 3 },
                status: Status::Success(("a", "b", "c"))
            }
        );
    }

    #[test]
    fn string_alternate() {
        fn any<'a>(d: &mut StringMaster<'a>, pt: StringPoint<'a>) -> StringProgress<'a, &'a str> {
            d.alternate(pt)
                .one(|_, pt| pt.consume_literal("a").map_err(|_| AnError(1)))
                .one(|_, pt| pt.consume_literal("b").map_err(|_| AnError(2)))
                .one(|_, pt| pt.consume_literal("c").map_err(|_| AnError(3)))
                .finish()
        }

        let mut d = ParseMaster::new();
        let pt = StringPoint::new("c");

        let r = any(&mut d, pt);
        let r = d.finish(r);

        assert_eq!(
            r,
            Progress {
                point: StringPoint { s: "", offset: 1 },
                status: Status::Success("c")
            }
        );
    }

    #[test]
    fn string_zero_or_more() {
        fn any<'a>(
            d: &mut StringMaster<'a>,
            pt: StringPoint<'a>,
        ) -> StringProgress<'a, Vec<&'a str>> {
            d.zero_or_more(pt, |_, pt| pt.consume_literal("a").map_err(|_| AnError(1)))
        }

        let mut d = ParseMaster::new();
        let pt = StringPoint::new("aaa");

        let r = any(&mut d, pt);
        let r = d.finish(r);

        assert_eq!(
            r,
            Progress {
                point: StringPoint { s: "", offset: 3 },
                status: Status::Success(vec!["a", "a", "a"])
            }
        );
    }

    #[test]
    fn string_to() {
        let pt1 = StringPoint::new("hello world");
        let pt2 = StringPoint {
            offset: pt1.offset + 5,
            s: &pt1.s[5..],
        };
        assert_eq!("hello", pt1.to(pt2));
    }

    #[test]
    fn string_consume_literal() {
        let pt = StringPoint::new("hello world");

        let r = pt.consume_literal("hello");
        assert_eq!(
            r,
            Progress {
                point: StringPoint {
                    s: " world",
                    offset: 5
                },
                status: Status::Success("hello")
            }
        );

        let r = pt.consume_literal("goodbye");
        assert_eq!(
            r,
            Progress {
                point: StringPoint {
                    s: "hello world",
                    offset: 0
                },
                status: Status::Failure(())
            }
        );
    }

    #[test]
    fn string_consume_identifier() {
        let pt = StringPoint::new("hello world");

        let r = pt.consume_identifier(&[("goodbye", 1), ("hello", 2)]);
        assert_eq!(
            r,
            Progress {
                point: StringPoint {
                    s: " world",
                    offset: 5
                },
                status: Status::Success(2)
            }
        );

        let r = pt.consume_identifier(&[("red", 3), ("blue", 4)]);
        assert_eq!(
            r,
            Progress {
                point: StringPoint {
                    s: "hello world",
                    offset: 0
                },
                status: Status::Failure(())
            }
        );
    }
}
