type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

pub trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;
    fn map<F, NewOutput>(self, map_fn: F) -> BoxedParser<'a, NewOutput>
    where
        Self: Sized + 'a,
        Output: 'a,
        NewOutput: 'a,
        F: Fn(Output) -> NewOutput + 'a,
    {
        BoxedParser::new(map(self, map_fn))
    }
    fn decide<F>(self, decide_fn: F) -> BoxedParser<'a, Output>
    where
        Self: Sized + 'a,
        Output: 'a,
        F: Fn(&Output) -> bool + 'a,
    {
        BoxedParser::new(decide(self, decide_fn))
    }
    fn and_then<F, NextParser, NewOutput>(self, f: F) -> BoxedParser<'a, NewOutput>
        where
            Self: Sized + 'a,
            Output: 'a,
            NewOutput: 'a,
            NextParser: Parser<'a, NewOutput> + 'a,
            F: Fn(Output) -> NextParser + 'a,
    {
        BoxedParser::new(and_then(self, f))
    }
    fn or<P2>(self, parser: P2) -> BoxedParser<'a, Output>
    where
        Self: Sized + 'a,
        Output: 'a,
        P2: Parser<'a, Output> + 'a,
    {
        BoxedParser::new(either(self, parser))
    }
}


impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) ->  ParseResult<'a, Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

pub struct BoxedParser<'a, O> {
    parser: Box<dyn Parser<'a, O> + 'a>
}

impl <'a, O> BoxedParser<'a, O> {
    fn new<P>(parser: P) -> Self
    where
        P: Parser<'a, O> + 'a,
    {
        BoxedParser {
            parser: Box::new(parser)
        }
    }
}

impl <'a, O> Parser<'a, O> for BoxedParser<'a, O> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, O> {
        self.parser.parse(input)
    }
}

fn decide<'a, P, O, F>(parser: P, decide_fn: F) -> impl Parser<'a, O>
    where
        P: Parser<'a, O>,
        F: Fn(&O) -> bool,
{
    move |input| match parser.parse(input) {
        Ok((rest, result)) if decide_fn(&result) => Ok((rest, result)),
        _ => Err(input),
    }
}

fn map<'a, P, F, Output, NewOutput>(parser: P, map_fn: F) -> impl Parser<'a, NewOutput>
where
    P: Parser<'a, Output>,
    F: Fn(Output) -> NewOutput
{
    move |input| parser.parse(input).map(|(rest, value)| (rest, map_fn(value)))
}

fn and_then<'a, P, O1, F, NextP, O2>(parser: P, f: F) -> impl Parser<'a, O2>
where
    P: Parser<'a, O1>,
    NextP: Parser<'a, O2>,
    F: Fn(O1) -> NextP,
{
    move |input| match parser.parse(input) {
        Ok((rest, result)) => f(result).parse(rest),
        _ => Err(input),
    }
}

fn either<'a, P1, P2, O>(p1: P1, p2: P2) -> impl Parser<'a, O>
where
    P1: Parser<'a, O>,
    P2: Parser<'a, O>,
{
    move |input| match p1.parse(input) {
        ok @ Ok(_) => ok,
        Err(_) => p2.parse(input),
    }
}

fn pair<'a, P1, P2, O1, O2>(p1: P1, p2: P2) -> impl Parser<'a, (O1, O2)>
where
    P1: Parser<'a, O1>,
    P2: Parser<'a, O2>,
{
    move |input| match p1.parse(input) {
        Ok((rest, result1)) => {
            p2.parse(rest).map(|(rest, result2)| (rest, (result1, result2)))
        },
        _ => Err(input),
    }
}


fn closure<'a, P, O>(parser: P) -> impl Parser<'a, Vec<O>>
where
    P: Parser<'a, O>,
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

fn match_literal<'a>(expected: &'static str) -> impl Parser<'a, ()> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => {
            let rest = &input[expected.len()..];
            Ok((rest, ()))
        },
        _ => Err(input),
    }
}

fn next_char(input: &str) -> ParseResult<char> {
    match input.chars().next() {
        Some(next) => Ok((&input[next.len_utf8()..], next)),
        _ => Err(input),
    }
}

fn identifier(input: &str) -> ParseResult<String> {
    pair(
        next_char.decide(
            |next| is_identifier_char(*next) && !next.is_ascii_digit()
        ),
        closure(
            next_char.decide(|next| is_identifier_char(*next))
        )
    ).map(|(first, mut chars)| {
        chars.insert(0, first);
        let result = chars.iter().fold(String::new(), |mut result, item| {
            result.push(*item);
            result
        });
        result
    }).parse(input)
}
fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c.eq(&'$') || c.eq(&'_')
}

fn match_number<'a>(input: &'a str) -> ParseResult<'a, String> {
    either(match_float, match_integer).parse(input)
}
fn match_integer<'a>(input: &'a str) -> ParseResult<'a, String> {
    closure(next_char.decide( |next| next.is_ascii_digit()))
        .map(
            |chars| chars.iter().fold(
                String::new(),
                |mut result, ch| {
                    result.push(*ch);
                    result
                }
            )
        ).parse(input)
}

fn match_float<'a>(input: &'a str) -> ParseResult<'a, String> {
    pair(match_integer, match_literal("."))
        .map(|(mut value, _)| {
            value.push('.');
            value
        })
        .and_then(|value| {
            match_integer.map(move |output| value.to_owned() + output.as_str())
        }).parse(input)
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
        let parser = next_char.decide(|value| value.is_alphabetic() || *value == '_' || *value == '$');
        assert_eq!(Ok(("i hao", 'n')), parser.parse("ni hao"), "decide pass testing");
    }

    #[test]
    fn it_closure() {
        let parser = closure(
            next_char.decide(|value| value.is_alphabetic() || *value == '_' || *value == '$')
        );
        assert_eq!(Ok((" hao", vec!['n', 'i'])), parser.parse("ni hao"), "decide pass testing");
    }

    #[test]
    fn it_identifier() {
        assert_eq!(
            Ok((" world!", String::from("Hello"))),
            identifier("Hello world!"),
        )
    }

    #[test]
    fn it_integer_number() {
        assert_eq!(
            Ok(("+3", String::from("2"))),
            match_number("2+3"),
        )
    }
    #[test]
    fn it_float_number() {
        assert_eq!(
            Ok(("+3", String::from("2.7"))),
            match_number("2.7+3"),
        )
    }
}
