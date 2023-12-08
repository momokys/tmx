use crate::parser::{next, Parser, sym};
use std::str;
use std::f64;
use std::str::FromStr;

pub struct Lexer<'a> {
    number: Parser<'a, u8, f64>,
    // identifier: Parser<'a, &'a str, String>
}

impl <'a> Lexer<'a> {
    fn new() -> Self {
        Self {
            number: Parser::comb(|| {
                let not_zero_ascii_digit = next::<u8>().decide(|&res| res.is_ascii_digit() && res != b'0');
                let ascii_digit = next::<u8>().decide(|&res| res.is_ascii_digit()).repeat(0..);
                let integer = not_zero_ascii_digit << ascii_digit | sym(b'0');

                let frac = sym(b'.') << next::<u8>().decide(|&res| res.is_ascii_digit()).repeat(1..);
                return (integer & frac.opt()).collect().convert(str::from_utf8).convert(f64::from_str);
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    #[test]
    fn it_number() {
        assert_eq!(
            Ok(22134f64),
            Lexer::new().number.parse(b"22134HD"),
        )
    }
}