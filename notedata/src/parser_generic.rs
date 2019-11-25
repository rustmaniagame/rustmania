use nom::{
    bytes::complete::{is_not, take_until},
    character::complete::{char, multispace0},
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

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
