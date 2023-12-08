use std::ops::{BitAnd, BitOr, Shl, Shr};
use crate::{Error, Result};
use crate::range::{Bound, RangeArgument};

type Parse<'a, I, O> = dyn Fn(&'a [I], usize) -> Result<(O, usize)> + 'a;
pub struct Parser<'a, I, O> {
    parse: Box<Parse<'a, I, O>>
}

impl <'a, I, O> Parser<'a, I, O> {
    pub fn new<P>(parse: P) -> Self
    where P: Fn(&'a [I], usize) -> Result<(O, usize)> + 'a
    {
        Self {
            parse: Box::new(parse)
        }
    }
    pub fn comb<F>(f: F) -> Parser<'a, I, O>
        where F: Fn() -> Parser<'a, I, O>
    {
        f()
    }
    pub fn parse(&self, input: &'a [I]) -> Result<O> {
        (self.parse)(input, 0).map(|(result, _)| result)
    }
    pub fn trace<F>(self, trace_fn: F) -> Parser<'a, I, O>
    where
    	O: 'a,
    	F: Fn((&O, usize, usize)) -> () + 'a
    {
        Parser::new(move |input: &'a [I], start| {
            match (self.parse)(input, start) {
                Ok((output, end)) => {
                    trace_fn((&output, start, end));
                    Ok((output, end))
                },
                _ => Err(Error::Incomplete)
            }
        })
    }
    pub fn map<F, U>(self, map_fn: F) -> Parser<'a, I, U>
    where
        O: 'a,
        F: Fn(O) -> U + 'a,
    {
        Parser::new(move |input, start| {
            (self.parse)(input, start).map(|(res, end)| ((map_fn(res), end)))
        })
    }
    pub fn collect(self) -> Parser<'a, I, &'a [I]>
    where O: 'a,
    {
        Parser::new(move |input, start| {
            (self.parse)(input, start).map(|(_, end)| (&input[start..end], end))
        })
    }
    pub fn convert<F, U, E>(self, convert_fn: F) -> Parser<'a, I, U>
    where
        O: 'a,
        F: Fn(O) -> std::result::Result<U, E> + 'a,
    {
        Parser::new(move |input, start| {
            (self.parse)(input, start).and_then(|(res, end)| {
                match convert_fn(res) {
                    Ok(res) => Ok((res, end)),
                    _ => Err(Error::Incomplete),
                }
            })
        })
    }
    pub fn decide<F>(self, decide_fn: F) -> Parser<'a, I, O>
    where
        O: 'a,
        F: Fn(&O) -> bool + 'a,
    {
        Parser::new(move |input, start| {
            match (self.parse)(input, start) {
                Ok((res, end)) if decide_fn(&res) => Ok((res, end)),
                _ => Err(Error::Incomplete),
            }
        })
    }
    pub fn repeat<R>(self, range: R) -> Parser<'a, I, Vec<O>>
    where
        R: RangeArgument<usize> + 'a,
        O: 'a,
    {
        Parser::new(move |input, mut start| {
            let mut index = 0;
            let mut end = start;
            let mut result = Vec::<O>::new();
            if let Bound::Included(0) = range.end() {
                return Ok((result, end));
            }
            loop {
                match (self.parse)(input, start) {
                    Ok((output, _end)) => {
                        result.push(output);
                        end = _end;
                    },
                    _ => {
                        end = start;
                        return match range.end() {
                            Bound::Unbounded => {
                                Ok((result, end))
                            },
                            _ => {
                                Err(Error::Incomplete)
                            }
                        }
                    }
                }
                start = end;
                index = index + 1;
                match range.end() {
                    Bound::Excluded(&max) if max <= index => {
                        return Ok((result, end));
                    },
                    Bound::Included(&max) if max < index => {
                        return Ok((result, end));
                    }
                    _ => {}
                }
            }
        })
    }
    pub fn opt(self) -> Parser<'a, I, Option<O>>
    where O: 'a
    {
        Parser::new(move |input, start| {
            match (self.parse)(input, start) {
                Ok((res, end)) => Ok((Some(res), end)),
                Err(_) => Ok((None, start)),
            }
        })
    }
}

pub fn epsilon<'a, I>() -> Parser<'a, I, ()> {
    Parser::new(|_: &'a [I], start| Ok(((), start)))
}

pub fn next<'a, I>() -> Parser<'a, I, I>
where I: Clone
{
    Parser::new(|input: &'a [I], start| {
        match input.iter().next() {
            Some(it) => Ok((it.clone(), start + 1)),
            None => Err(Error::Incomplete),
        }
    })
}

pub fn sym<'a, I>(t: I) -> Parser<'a, I, I>
where I: PartialEq + Clone
{
    Parser::new(move |input: &'a [I], start| {
        match input.get(start) {
            Some(s) if *s == t => Ok((s.clone(), start + 1)),
            _ => Err(Error::Incomplete),
        }
    })
}

pub fn seq<'a, 'b: 'a, I>(t: &'b [I]) -> Parser<'a, I, &'b [I]>
where I: PartialEq
{
    Parser::new(move |input, start| {
        let mut result = vec![];
        for index in 0..t.len() {
            match input.get(start + index) {
                Some(s) if *s == *t.get(index).unwrap() => result.push(t.clone()),
                _ => return Err(Error::Incomplete),
            };
        }
        Ok((t, start + t.len()))
    })
}

impl <'a, I, O: 'a, U: 'a> BitAnd<Parser<'a, I, U>> for Parser<'a, I, O> {
    type Output = Parser<'a, I, (O, U)>;

    fn bitand(self, rhs: Parser<'a, I, U>) -> Self::Output {
        Parser::new(move |input, start| {
            (self.parse)(input, start).and_then(
                |(o, end)| (rhs.parse)(input, end).map(|(u, end)| ((o,u), end))
            )
        })
    }
}
impl <'a, I, O: 'a> BitOr for Parser<'a, I, O> {
    type Output = Parser<'a, I, O>;

    fn bitor(self, rhs: Self) -> Self::Output {
        Parser::new(move |input, start| {
            match (self.parse)(input, start) {
                Ok((res, end)) => Ok((res, end)),
                _ => (rhs.parse)(input, start),
            }
        })
    }
}

impl <'a, I, O: 'a, U: 'a> Shl<Parser<'a, I, U>> for Parser<'a, I, O> {
    type Output = Parser<'a, I, O>;

    fn shl(self, rhs: Parser<'a, I, U>) -> Self::Output {
        Parser::new(move |input, start| {
            (self.parse)(input, start).and_then(
                |(res, end)| (rhs.parse)(input, end).map(|(_, end)| (res, end))
            )
        })
    }
}

impl <'a, I, O: 'a, U: 'a> Shr<Parser<'a, I, U>> for Parser<'a, I, O> {
    type Output = Parser<'a, I, U>;

    fn shr(self, rhs: Parser<'a, I, U>) -> Self::Output {
        Parser::new(move |input, start| {
            (self.parse)(input, start).and_then(
                |(_, end)| (rhs.parse)(input, end).map(|(res, end)| (res, end))
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::*;
    use std::str;

    #[test]
    fn it_epsilon() {
        assert_eq!(
            Ok(()),
            epsilon().parse(b"Hello world!")
        )
    }
    #[test]
    fn it_next() {
        assert_eq!(
            Ok(b'H'),
            next().parse(b"Hello world"),
        )
    }
    #[test]
    fn it_sym() {
        assert_eq!(
            Ok(b'H'),
            sym(b'H').trace(|(res, _, _)| println!("{}", *res)).parse(b"Hello world!")
        )
    }
    #[test]
    fn it_seq() {
        assert_eq!(
            Ok("Hello"),
            seq(b"Hello").convert(str::from_utf8).parse(b"Hello world!"),
        )
    }
    #[test]
    fn it_repeat() {
        assert_eq!(
            Ok("H"),
            next::<u8>()
                .decide(|&res| res.is_ascii_alphabetic())
                .repeat(0..)
                .collect()
                .convert(str::from_utf8)
                .parse(b"He"),
        )
    }

    #[test]
    fn it_ops_bit_and() {
        assert_eq!(
            Ok((b'H', b'e')),
            (sym(b'H') & sym(b'e')).parse(b"Hello world"),
        )
    }

    #[test]
    fn it_ops_bit_or() {
        assert_eq!(
            Ok(b'H'),
            (sym(b'H') | sym(b'h')).parse(b"Hello world"),
        )
    }
    #[test]
    fn it_ops_shl() {
        assert_eq!(
            Ok(b'H'),
            (sym(b'H') << sym(b'e')).parse(b"Hello world"),
        )
    }
    #[test]
    fn it_ops_shr() {
        assert_eq!(
            Ok(b'e'),
            (sym(b'H') >> sym(b'e')).parse(b"Hello world"),
        )
    }
}