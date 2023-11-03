type ParserResult<Input, Output> = Result<(Input, Output), Input>;

pub trait Parser<Input, Output> {
    fn parse(&self, input: Input) -> ParserResult<Input, Output>;
}


impl<'a, F, Output> Parser<&'a str, Output> for F
where
    F: Fn(&'a str) ->  ParserResult<&'a str, Output>,
{
    fn parse(&self, input: &'a str) -> ParserResult<&'a str, Output> {
        self(input)
    }
}

fn decide<'a, P, O, F>(parser: P, decide_fn: F) -> impl Parser<&'a str, O>
    where
        P: Parser<&'a str, O>,
        F: Fn(&O) -> bool,
{
    move |input| match parser.parse(input) {
        Ok((rest, result)) if decide_fn(&result) => Ok((rest, result)),
        _ => Err(input),
    }
}

fn add_then<'a, P, O1, F, NextP, O2>(parser: P, f: F) -> impl Parser<&'a str, O2>
where
    P: Parser<&'a str, O1>,
    NextP: Parser<&'a str, O2>,
    F: Fn(O1) -> NextP,
{
    move |input| match parser.parse(input) {
        Ok((rest, result)) => f(result).parse(rest),
        _ => Err(input),
    }
}

fn pair<'a, P1, P2, O1, O2>(p1: P1, p2: P2) -> impl Parser<&'a str, (O1, O2)>
where
    P1: Parser<&'a str, O1>,
    P2: Parser<&'a str, O2>,
{
    move |input| match p1.parse(input) {
        Ok((rest, result1)) => {
            p2.parse(rest).map(|(rest, result2)| (rest, (result1, result2)))
        },
        _ => Err(input),
    }
}

fn zero_or_more<'a, P, O>(parser: P) -> impl Parser<&'a str, Vec<O>>
where
    P: Parser<&'a str, O>,
{
    move |mut input| {
        let mut result = Vec::new();
        while let Ok((rest, value)) = parser.parse(input) {
            result.push(value);
            input = rest;
        }
        Ok((input, result))
    }
}

fn one_or_more<'a, P, O>(parser: P) -> impl Parser<&'a str, Vec<O>>
    where
        P: Parser<&'a str, O>,
{
    decide(
        zero_or_more(parser),
        |result| result.len() > 0,
    )
}

fn match_literal<'a>(expected: &'static str) -> impl Parser<&'a str, ()> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => {
            let rest = &input[expected.len()..];
            Ok((rest, ()))
        },
        _ => Err(input),
    }
}

fn next_char(input: &str) -> ParserResult<&str, char> {
    match input.chars().next() {
        Some(next) => Ok((&input[next.len_utf8()..], next)),
        _ => Err(input),
    }
}

fn identifier(input: &str) -> ParserResult<&str, String> {
    pair(
        decide(
            next_char,
            |next| next.is_alphabetic(),
        ),
        zero_or_more(
            decide(
                next_char,
                |next| next.is_alphabetic() || next.is_ascii_digit()
            )
        )
    ).parse(input).map(|(rest, (first, mut chars))| {
        chars.insert(0, first);
        let mut result = String::new();
        for char in chars {
            result.push(char);
        }
        (rest, result)
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_match_literal() {
        let parser = match_literal("ni");
        assert_eq!(Ok((" hao", ())), parser.parse("ni hao"), "match_literal pass testing");
    }
    #[test]
    fn it_decide() {
        let parser = decide(next_char, |value| value.is_alphabetic() || *value == '_' || *value == '$');
        assert_eq!(Ok(("i hao", 'n')), parser.parse("ni hao"), "decide pass testing");
    }

    #[test]
    fn it_zero_or_more() {
        let parser = zero_or_more(
            decide(
                next_char,
                |value| value.is_alphabetic() || *value == '_' || *value == '$'
            )
        );
        assert_eq!(Ok((" hao", vec!['n', 'i'])), parser.parse("ni hao"), "decide pass testing");
    }

    #[test]
    fn it_identifier() {
        assert_eq!(Ok((" world!", String::from("Hello"))), identifier("Hello world!"))
    }
}
