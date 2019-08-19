use nom::IResult;
use nom::branch::alt;
use nom::sequence::{tuple, delimited};
use nom::number::complete::double;
use crate::lex::{identifier, Identify};
use nom::character::complete::{char, space0};
use nom::combinator::{map, recognize};
use nom::multi::{many0, many1};
use nom::bytes::complete::{tag, take_until};

pub type Expression = String;

fn brackets(input: &str) -> IResult<&str, Expression> {
    map(delimited(char('('), expression, char(')')),
        |matched: String| "(".to_owned() + matched.as_str() + ")",
    )(input)
}

fn string_literal(input: &str) -> IResult<&str, Expression> {
    map(recognize(delimited(
        char('"'),
        take_until("\""),
        char('"'),
    )), |it: &str| it.to_owned())(input)
}

type StructField = String;

fn struct_field(input: &str) -> IResult<&str, StructField> {
    map(tuple((
        identifier,
        many0(tuple((char('.'), identifier)))
    )), |matched: (Identify, Vec<(_, Identify)>)| {
        matched.1.iter()
            .map(|it| it.1.to_owned())
            .fold(matched.0, |acc, this| acc + ":" + this.as_str())
    })(input)
}

fn double_str(input: &str) -> IResult<&str, String> {
    map(recognize(double), |it: &str| it.to_owned())(input)
}

fn higher_than_factor(input: &str) -> IResult<&str, Expression> {
    alt((brackets, function_call, struct_field, double_str, identifier))(input)
}

fn factor(input: &str) -> IResult<&str, Expression> {
    alt((
        map(tuple((
            higher_than_factor,
            many1(tuple((
                space0,
                alt((char('*'), char('/'), char('^'))),
                space0,
                higher_than_factor))),
        )), |matched: (String, Vec<(_, char, _, String)>)| {
            matched.1.iter().fold(matched.0, |acc, current| acc + current.1.to_string().as_str() + current.3.as_str())
        }),
        higher_than_factor
    ))(input)
}

fn higher_than_term(input: &str) -> IResult<&str, String> {
    factor(input)
}

pub fn term(input: &str) -> IResult<&str, Expression> {
    alt((
        map(tuple((
            higher_than_term,
            many1(tuple((
                space0,
                alt((char('+'), char('-'))),
                space0,
                higher_than_term))),
        )), |matched: (String, Vec<(_, char, _, String)>)| {
            matched.1.iter().fold(matched.0, |acc, current| acc + current.1.to_string().as_str() + current.3.as_str())
        }),
        higher_than_term
    ))(input)
}

fn higher_than_relation(input: &str) -> IResult<&str, Expression> {
    term(input)
}

pub fn relation(input: &str) -> IResult<&str, Expression> {
    map(tuple((
        higher_than_relation,
        space0,
        alt((
            tag("<="), tag(">="),
            tag("<"), tag(">"),
            tag("=="), tag("!="), tag("<>")
        )),
        space0,
        higher_than_relation
    )), |(left, _, op, _, right)| {
        let real_op = match op {
            "==" => "=",
            "!=" => "<>",
            _ => op
        };
        left + real_op + right.as_str()
    })(input)
}

fn not_logic(input: &str) -> IResult<&str, Expression> {
    map(tuple((
        char('!'),
        alt((logic, relation, brackets))
    )), |(_, op_num)| { "not ".to_string() + op_num.as_str() })(input)
}

pub fn logic(input: &str) -> IResult<&str, Expression> {
    alt((
        map(tuple((
            alt((not_logic, relation)),
            many1(tuple((
                space0,
                alt((
                    tag("and"), tag("&&"),
                    tag("or"), tag("||")
                )),
                space0,
                alt((not_logic, relation)))))
        )), |(acc, rest): (String, Vec<(_, &str, _, String)>)| {
            rest.iter().fold(acc, |acc, (_, op, _, num)| {
                let real_op = match op.to_owned() {
                    "&&" => "and",
                    "||" => "or",
                    _ => op
                };
                acc + " " + real_op + " " + num.as_str()
            })
        }),
        not_logic
    ))(input)
}

pub fn param_list(input: &str) -> IResult<&str, Vec<Expression>> {
    alt((
        map(tuple((
            expression,
            many0(tuple((space0, char(','), space0, expression))),
        )), |(first, rest): (Expression, Vec<(_, _, _, Expression)>)| {
            vec![first].into_iter().chain(rest.into_iter().map(|it| it.3)).collect()
        }),
        map(space0, |_| { vec![] })))(input)
}

fn function_call(input: &str) -> IResult<&str, Expression> {
    map(tuple((
        alt((identifier, struct_field)),
        delimited(
            char('('),
            param_list,
            char(')'),
        )
    )), |(function_name, function_params): (String, Vec<Expression>)| {
        function_name + "(" + function_params.join(",").as_str() + ")"
    })(input)
}

pub fn expression(input: &str) -> IResult<&str, String> {
    alt((logic, relation, term, factor, function_call, struct_field, brackets, string_literal))(input)
}

