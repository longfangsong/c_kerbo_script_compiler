use nom::sequence::tuple;
use nom::IResult;
use nom::character::complete::{alpha1, alphanumeric0};
use nom::combinator::map;

pub type Identify = String;

pub fn identifier(input: &str) -> IResult<&str, Identify> {
    map(tuple((
        alpha1,
        alphanumeric0
    )), |result: (&str, &str)| {
        result.0.to_owned() + result.1
    })(input)
}