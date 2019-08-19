extern crate nom;

mod lex;
mod expression;
mod statement;

use crate::statement::{Statement, compound};
use std::io;
use std::io::Read;


fn main() {
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("{}", compound(buffer.as_str())
        .map(|(_, out)| out.generate_code()).unwrap())
}