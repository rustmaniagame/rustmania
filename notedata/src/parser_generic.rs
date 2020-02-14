use crate::{lib::Vec, BeatPair};
use nom::{
    bytes::complete::{is_not, take_until},
    character::complete::{char, multispace0},
    combinator::{map, map_opt},
    multi::separated_nonempty_list,
    number::complete::double,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

pub fn comma_separated<'a, P, O>(parser: P) -> impl Fn(&'a str) -> IResult<&str, Vec<O>>
where
    P: Fn(&'a str) -> IResult<&str, O>,
{
    move |input: &str| separated_nonempty_list(ws_trimmed(char(',')), &parser)(input)
}

pub fn beat_pair<'a, P, O>(parser: P, scale: f64) -> impl Fn(&'a str) -> IResult<&str, BeatPair<O>>
where
    P: Fn(&'a str) -> IResult<&str, O>,
{
    move |input: &'a str| {
        map_opt(
            separated_pair(map(double, |x| x / scale), ws_trimmed(char('=')), &parser),
            |(beat, value)| BeatPair::from_pair(beat, value),
        )(input)
    }
}

pub fn ws_trimmed<'a, P, O>(parser: P) -> impl Fn(&'a str) -> IResult<&str, O>
where
    P: Fn(&'a str) -> IResult<&str, O>,
{
    move |input: &str| preceded(multispace0, terminated(&parser, multispace0))(input)
}

pub fn stepmania_tag(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(
        preceded(char('#'), ws_trimmed(is_not(": \t\r\n"))),
        char(':'),
        terminated(take_until(";"), char(';')),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_beat_pair() {
        let parsed_beat_pair = BeatPair::from_pair(123.456 / 4.0, 654.321).unwrap();
        assert_eq!(
            beat_pair(double, 4.0)("123.456  = 654.321  foo"),
            Ok(("  foo", parsed_beat_pair.clone()))
        );
        assert_eq!(
            beat_pair(double, 4.0)("123.456=654.321foo"),
            Ok(("foo", parsed_beat_pair))
        );
    }

    #[test]
    fn parse_sm_tag() {
        assert_eq!(
            stepmania_tag("# foo  : bar  ;  baz"),
            Ok(("  baz", ("foo", " bar  ")))
        );
        assert_eq!(stepmania_tag("#foo:bar;baz"), Ok(("baz", ("foo", "bar"))));
    }
}
