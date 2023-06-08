mod ast;
mod eval;
mod lexer;
mod parser;

use eval::EvalError;
use parser::{parse, ParseError};
use std::collections::HashMap;
use std::io;
use std::io::BufRead;

use crate::ast::Value;

#[derive(Debug)]
enum Error {
    Parse(ParseError),
    Eval(EvalError),
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Error::Parse(value)
    }
}

impl From<EvalError> for Error {
    fn from(value: EvalError) -> Self {
        Error::Eval(value)
    }
}

fn main() -> Result<(), Error> {
    println!("Hello, world!");

    let stdin = io::stdin();
    let s = stdin
        .lock()
        .lines()
        .map(|x| x.unwrap() + "\n")
        .collect::<Vec<String>>()
        .concat();
    let tokens = lexer::tokenize(&s);
    println!("{:?}", tokens);
    println!("I parsed:");

    let object = parse(&tokens)?;
    if let Value::Object(ref records) = object {
        println!("{:?}", records);

        for ele in records {
            println!("\t\"{}\": {:?}", ele.id, ele.value);
        }
    }

    println!("== Doing Eval ==");

    let call: eval::ValueCallFn = |x| match x {
        Value::String(s) => {
            let result = s.parse::<i32>();
            match result {
                Ok(num) => Ok(Value::Number(num)),
                Err(err) => Err(EvalError::Eval(Box::new(err))),
            }
        }
        _ => Ok(x.clone()),
    };

    let mut functions = HashMap::new();
    functions.insert("call".to_string(), call);

    let value = eval::eval(&object, &functions)?;
    println!("{:?}", value);

    Ok(())
}
