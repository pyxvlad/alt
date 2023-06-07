pub mod lexer;

use core::fmt;
use std::io;
use std::io::BufRead;
use std::iter::Peekable;

#[derive(Debug, Clone)]
enum ParseError {
    EndOfInput,
    ExpectedIdentifier,
    ExpectedValue,
    ExpectedAssign,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::EndOfInput => write!(f, "reached end of input while expecting more"),
            Self::ExpectedIdentifier => write!(f, "expected identifier"),
            Self::ExpectedValue => write!(f, "expected value"),
            Self::ExpectedAssign => write!(f, "expected assignment"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Value {
    Number(i32),
}

#[derive(Debug, PartialEq)]
struct Record {
    id: String,
    value: Value,
}

fn parse_value<'a, T>(it: &mut Peekable<T>) -> Result<Value, ParseError>
where
    T: Iterator<Item = &'a lexer::Token>,
{
    match it.peek() {
        None => return Err(ParseError::EndOfInput),
        Some(token) => match token {
            lexer::Token::Number(num) => {
                return Ok(Value::Number(*num));
            }
            _ => return Err(ParseError::ExpectedValue),
        },
    };
}

fn parse_record<'a, T>(it: &mut Peekable<T>) -> Result<Record, ParseError>
where
    T: Iterator<Item = &'a lexer::Token>,
{
    match it.peek() {
        None => return Err(ParseError::EndOfInput),
        Some(token) => match token {
            lexer::Token::ID(id) => {
                it.next();

                if let Some(token) = it.peek() {
                    if **token == lexer::Token::Assign {
                        it.next();

                        let value = parse_value(it)?;

                        return Ok(Record {
                            id: id.to_owned(),
                            value,
                        });
                    } else {
                        return Err(ParseError::ExpectedAssign);
                    }
                } else {
                    return Err(ParseError::EndOfInput);
                }
            }
            _ => return Err(ParseError::ExpectedIdentifier),
        },
    }
}

fn parse(tokens: &Vec<lexer::Token>) -> Result<Vec<Record>, ParseError> {

    let mut records: Vec<Record> = Vec::new();
    let mut it = tokens.iter().peekable();
    loop {
        match it.peek() {
            None => break,
            Some(token) => match token {
                lexer::Token::ID(_) => {
                    let record = parse_record(&mut it)?;
                    records.push(record);
                }
                lexer::Token::Separator => (),
                _ => todo!("implement it for {:?}", token),
            },
        };

        it.next();
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_number_value() -> Result<(), ParseError> {
        let tokens = lexer::tokenize("42");

        let mut it = tokens.iter().peekable();
        let value = parse_value(&mut it)?;
        assert_eq!(value, Value::Number(42));

        Ok(())
    }

    #[test]
    fn test_parse_record() -> Result<(), ParseError> {
        let tokens = lexer::tokenize("x = 2");

        let mut it = tokens.iter().peekable();
        let record = parse_record(&mut it)?;

        assert_eq!(
            record,
            Record {
                id: "x".to_string(),
                value: Value::Number(2),
            }
        );

        Ok(())
    }
}

fn main() -> Result<(), ParseError> {
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

    let records =  parse(&tokens)?;
    println!("{:?}", records);



    for ele in records {
        println!("\t\"{}\": {:?}", ele.id, ele.value);
    }

    Ok(())
}
