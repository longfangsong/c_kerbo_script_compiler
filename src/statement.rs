use crate::lex::{Identify, identifier};
use crate::expression::{Expression, expression, param_list, logic, relation};
use nom::IResult;
use nom::combinator::{map, opt};
use nom::sequence::{tuple, delimited};
use nom::bytes::complete::tag;
use nom::character::complete::{char, space1, space0, line_ending};
use nom::multi::many0;
use nom::branch::alt;

pub trait Statement {
    fn generate_code(&self) -> String;
}

#[derive(Debug)]
pub struct VariableDeclaration {
    identifier: Identify,
    init_value: Option<Expression>,
}

impl Statement for VariableDeclaration {
    fn generate_code(&self) -> String {
        let init = self.init_value.to_owned().unwrap_or("0".to_string());
        "DECLARE ".to_string() + self.identifier.as_str() + " TO " + init.as_str() + "."
    }
}

pub fn variable_declare(input: &str) -> IResult<&str, VariableDeclaration> {
    map(tuple((
        tag("var"),
        space1,
        identifier,
        opt(tuple((
            space0,
            char('='),
            space0,
            expression
        ))),
        char(';')
    )), |result: (_, _, Identify, Option<(_, _, _, Expression)>, _)| {
        VariableDeclaration {
            identifier: result.2,
            init_value: result.3.map(|it| { it.3 }),
        }
    })(input)
}

#[derive(Debug)]
pub struct LockStatement {
    identifier: Identify,
    value: Expression,
}

impl Statement for LockStatement {
    fn generate_code(&self) -> String {
        "LOCK ".to_string() + self.identifier.as_str() + " TO " + self.value.as_str() + "."
    }
}

#[derive(Debug)]
pub struct VariableAssign {
    identifier: Identify,
    value: Expression,
}

impl Statement for VariableAssign {
    fn generate_code(&self) -> String {
        "SET ".to_string() + self.identifier.as_str() + " to " + self.value.as_str() + "."
    }
}

pub fn variable_assign(input: &str) -> IResult<&str, VariableAssign> {
    map(tuple((
        identifier,
        space0,
        char('='),
        space0,
        expression,
        char(';')
    )), |(identifier, _, _, _, value, _)| {
        VariableAssign {
            identifier,
            value,
        }
    })(input)
}


pub fn lock(input: &str) -> IResult<&str, LockStatement> {
    map(tuple((
        tag("assign"),
        space1,
        identifier,
        space0,
        char('='),
        space0,
        expression,
        char(';')
    )), |(_, _, identifier, _, _, _, value, _)| {
        LockStatement {
            identifier,
            value,
        }
    })(input)
}

#[derive(Debug)]
pub struct PrintStatement {
    values: Vec<Expression>,
}

pub fn print(input: &str) -> IResult<&str, PrintStatement> {
    map(tuple((
        tag("print"),
        delimited(
            char('('),
            param_list,
            char(')'),
        ),
        char(';')
    )), |(_, params, _)| {
        PrintStatement { values: params }
    })(input)
}

impl Statement for PrintStatement {
    fn generate_code(&self) -> String {
        "print ".to_string() + self.values.join("+").as_str() + "."
    }
}


pub fn statement(input: &str) -> IResult<&str, Box<dyn Statement>> {
    map(tuple((
        space0,
        alt((
            map(variable_declare, |it| -> Box<dyn Statement> {
                Box::new(it)
            }),
            map(variable_assign, |it| -> Box<dyn Statement> {
                Box::new(it)
            }),
            map(lock, |it| -> Box<dyn Statement> {
                Box::new(it)
            }),
            map(print, |it| -> Box<dyn Statement> {
                Box::new(it)
            }),
            map(while_statement, |it| -> Box<dyn Statement> {
                Box::new(it)
            }),
            map(for_statement, |it| -> Box<dyn Statement> {
                Box::new(it)
            })
        ))
        , space0)
    ), |(_, result, _)| result)(input)
}

pub struct CompoundStatement {
    values: Vec<Box<dyn Statement>>,
}

impl Statement for CompoundStatement {
    fn generate_code(&self) -> String {
        self.values.iter().map(|it| {
            it.generate_code()
        }).collect::<Vec<String>>().join("\n")
    }
}

pub fn compound(input: &str) -> IResult<&str, CompoundStatement> {
    map(
        many0(tuple((statement, opt(line_ending)))),
        |matched: Vec<(Box<dyn Statement>, _)>| CompoundStatement { values: matched.into_iter().map(|it| it.0).collect() },
    )(input)
}

pub struct WhileStatement {
    condition: Expression,
    body: CompoundStatement,
}

impl Statement for WhileStatement {
    fn generate_code(&self) -> String {
        return "UNTIL not (".to_string() + self.condition.as_str() + ") {" + self.body.generate_code().as_str() + "\n}";
    }
}

pub fn while_statement(input: &str) -> IResult<&str, WhileStatement> {
    map(tuple((
        tag("while"),
        space1,
        expression,
        space1,
        char('{'),
        alt((space0, line_ending)),
        compound,
        alt((space0, line_ending)),
        char('}')
    )), |(_, _, condition, _, _, _, body, _, _)| { WhileStatement { condition, body } })(input)
}

pub struct ForStatement {
    init: VariableDeclaration,
    condition: Expression,
    step: VariableAssign,
    body: CompoundStatement,
}

impl Statement for ForStatement {
    fn generate_code(&self) -> String {
        return "FROM {".to_string() + self.init.generate_code().as_str() + "} UNTIL not("
            + self.condition.as_str() + ") STEP {"
            + self.step.generate_code().as_str() + "} DO {\n"
            + self.body.generate_code().as_str() + "\n}";
    }
}

pub fn for_statement(input: &str) -> IResult<&str, ForStatement> {
    println!("{}", input);
    map(tuple((
        tag("for"),
        space0,
        variable_declare,
        space0,
        alt((logic, relation)),
        char(';'),
        space0,
        identifier,
        space0,
        char('='),
        space0,
        expression,
        space1,
        char('{'),
        alt((line_ending, space0)),
        compound,
        alt((space0, line_ending)),
        char('}')
    )), |(_, _, init, _, condition, _, _, identifier, _, _, _, value, _, _, _, body, _, _)| {
        ForStatement {
            init,
            condition,
            step: VariableAssign {
                identifier,
                value,
            },
            body,
        }
    })(input)
}