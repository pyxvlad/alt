#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cargo_common_metadata)]

use alt::ast::Record;
use alt::ast::Value;
use alt::eval;
use alt::eval::Evaluator;
use alt::goodies;
use alt::lexer;
use alt::parser;
use alt::parser::parse;
use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use std::io::BufRead;

#[derive(Debug)]
enum Error {
    Parse(parser::Error),
    Eval(eval::Error),
    SerdeJson(serde_json::Error),
    NotName,
    NotNumber,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "parser error: {err}"),
            Self::Eval(err) => write!(f, "evaluation error: {err}"),
            Self::SerdeJson(err) => write!(f, "serde_json error: {err}"),
            Self::NotName => write!(f, "not a possible cat name"),
            Self::NotNumber => write!(f, "not a number"),
        }
    }
}

impl std::error::Error for Error {}

impl From<parser::Error> for Error {
    fn from(value: parser::Error) -> Self {
        Self::Parse(value)
    }
}

impl From<eval::Error> for Error {
    fn from(value: eval::Error) -> Self {
        Self::Eval(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
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
    println!("I parsed:");

    let object = parse(&tokens).or_else(|e| {
        let (left, _) = s.split_at(e.pos.start);
        let line = left.lines().count() - 1;
        if let Some(l) = s.lines().nth(line) {
            println!("Syntax error:");
            println!("\t{l}");

            println!("\t{}^ {e}", " ".repeat(e.pos.start));
        }

        Err(e)
    })?;
    if let Value::ObjectWithCalls(ref records) = object {
        println!("{records:?}");

        for ele in records {
            println!("\t{:?}", ele);
        }
    }

    println!("== Doing Eval ==");

    struct T {
        good_evaluator: goodies::Evaluator,
    }

    impl T {
        fn call(x: &Value) -> Result<Value, eval::Error> {
            match x {
                Value::String(s) => {
                    let result = s.parse::<i32>();
                    match result {
                        Ok(num) => Ok(Value::Number(num)),
                        Err(err) => Err(eval::Error::Eval(Box::new(err))),
                    }
                }
                _ => Ok(x.clone()),
            }
        }
        fn pisoi(x: &Value) -> Result<Value, eval::Error> {
            match x {
                Value::String(_) => Ok(Value::Typed(alt::ast::Typed {
                    kind: "pisoi".to_string(),
                    value: Box::new(x.clone()),
                })),
                _ => Err(eval::Error::Eval(Box::new(Error::NotName))),
            }
        }
        fn itoa(x: &Value) -> Result<Value, eval::Error> {
            match x {
                Value::Number(n) => Ok(Value::String(n.to_string())),
                _ => Err(eval::Error::Eval(Box::new(Error::NotNumber))),
            }
        }

        fn pisoi_record(x: &Value) -> Result<Option<Record>, eval::Error> {
            match x {
                Value::String(s) => Ok(Some(alt::ast::Record {
                    id: s.clone(),
                    value: Value::Typed(alt::ast::Typed {
                        kind: "pisoi".to_string(),
                        value: Box::new(x.clone()),
                    }),
                })),
                _ => Err(eval::Error::Eval(Box::new(Error::NotName))),
            }
        }
    }

    impl Evaluator for T {
        fn value_function_eval(&mut self, call: &alt::ast::Call) -> Result<Value, eval::Error> {
            match call.function.as_str() {
                "call" => Self::call(&call.value),
                "pisoi" => Self::pisoi(&call.value),
                "itoa" => Self::itoa(&call.value),
                _ => self.good_evaluator.value_function_eval(call),
            }
        }

        fn record_function_eval(
            &mut self,
            call: &alt::ast::Call,
        ) -> Result<Option<Record>, eval::Error> {
            match call.function.as_str() {
                "pisoi" => Self::pisoi_record(&call.value),
                _ => self.good_evaluator.record_function_eval(call),
            }
        }
    }

    let mut ev = T {
        good_evaluator: Default::default(),
    };

    let result = ev.eval(&object);
    let value;

    println!("{result:?}");

    match result {
        Ok(t) => value = t,
        Err(err) => {
            panic!("{err}")
        }
    }

    println!("{value:?}");

    println!("JSON Value:");

    let json = serde_json::to_string_pretty(&value)?;

    println!("{json}");

    Ok(())
}
