use crate::ast::{Call, Record, Value};
use crate::lexer::{self, FilePos};
use std::error::Error as StdError;
use std::fmt;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum ErrorTypes {
    EndOfInput,
    ExpectedIdentifier,
    ExpectedValue,
    ExpectedAssign,
    ExpectedNumber,
}

impl fmt::Display for ErrorTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::EndOfInput => write!(f, "reached end of input while expecting more"),
            Self::ExpectedIdentifier => write!(f, "expected identifier"),
            Self::ExpectedValue => write!(f, "expected value"),
            Self::ExpectedAssign => write!(f, "expected assignment"),
            Self::ExpectedNumber => write!(f, "expected number"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub error: ErrorTypes,
    pub pos: FilePos,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.error, self.pos)
    }
}

impl StdError for Error {}

fn parse_value<'a, T>(it: &mut Peekable<T>) -> Result<Value, Error>
where
    T: Iterator<Item = &'a lexer::Token>,
{
    match it.peek() {
        None => Err(Error {
            error: ErrorTypes::EndOfInput,
            pos: Default::default(),
        }),
        Some(token) => match &token.kind {
            lexer::TokenKind::Number(num) => {
                it.next();
                if let Some(token) = it.peek() {
                    if token.kind == lexer::TokenKind::Dot {
                        let token = token.clone();
                        it.next();
                        if let Some(token) = it.peek() {
                            match token.kind {
                                lexer::TokenKind::Number(n) => {
                                    // see https://stackoverflow.com/a/69298721
                                    let decimals = n.checked_ilog10().unwrap_or(0) + 1;

                                    let x = 10.0_f32
                                        .powi(-(decimals as i32))
                                        .mul_add(n as f32, *num as f32);
                                    return Ok(Value::Float(x));
                                }
                                _ => {
                                    return Err(Error {
                                        error: ErrorTypes::ExpectedNumber,
                                        pos: token.pos,
                                    });
                                }
                            }
                        }
                        return Err(Error {
                            error: ErrorTypes::EndOfInput,
                            pos: token.pos,
                        });
                    }
                }
                Ok(Value::Number(*num))
            }
            lexer::TokenKind::String(s) => Ok(Value::String(s.clone())),
            lexer::TokenKind::LeftBrace => {
                it.next();
                let records = parse_multiple_records(it, &lexer::TokenKind::RightBrace)?;
                Ok(Value::Object(records))
            }
            lexer::TokenKind::ValueCall => {
                it.next();
                let call = parse_value_call(it)?;
                Ok(Value::Call(call))
            }
            _ => Err(Error {
                error: ErrorTypes::ExpectedValue,
                pos: token.pos,
            }),
        },
    }
}

fn parse_record<'a, T>(it: &mut Peekable<T>) -> Result<Record, Error>
where
    T: Iterator<Item = &'a lexer::Token>,
{
    match it.peek() {
        None => Err(Error {
            error: ErrorTypes::EndOfInput,
            pos: Default::default(),
        }),
        Some(token) => match &token.kind {
            lexer::TokenKind::ID(id) => {
                let token = token.clone();
                it.next();

                if let Some(token) = it.peek() {
                    if token.kind == lexer::TokenKind::Assign {
                        it.next();

                        let value = parse_value(it)?;

                        Ok(Record {
                            id: id.clone(),
                            value,
                        })
                    } else {
                        Err(Error {
                            error: ErrorTypes::ExpectedAssign,
                            pos: token.pos,
                        })
                    }
                } else {
                    Err(Error {
                        error: ErrorTypes::EndOfInput,
                        pos: token.pos,
                    })
                }
            }
            _ => Err(Error {
                error: ErrorTypes::ExpectedIdentifier,
                pos: token.pos,
            }),
        },
    }
}

fn parse_multiple_records<'a, T>(
    it: &mut Peekable<T>,
    end: &lexer::TokenKind,
) -> Result<Vec<Record>, Error>
where
    T: Iterator<Item = &'a lexer::Token>,
{
    let mut records: Vec<Record> = Vec::new();
    loop {
        match it.peek() {
            None => {
                return Err(Error {
                    error: ErrorTypes::EndOfInput,
                    pos: Default::default(),
                })
            }
            Some(token) => match token.kind {
                lexer::TokenKind::ID(_) => {
                    let record = parse_record(it)?;
                    records.push(record);
                }
                lexer::TokenKind::Separator => (),
                _ if *end == token.kind => break,
                _ => todo!("{:?}", token),
            },
        };

        it.next();
    }

    Ok(records)
}

fn parse_value_call<'a, T>(it: &mut Peekable<T>) -> Result<Call, Error>
where
    T: Iterator<Item = &'a lexer::Token>,
{
    match it.peek() {
        None => Err(Error {
            error: ErrorTypes::EndOfInput,
            pos: Default::default(),
        }),
        Some(token) => match &token.kind {
            lexer::TokenKind::ID(function) => {
                it.next();
                let value = parse_value(it)?;
                Ok(Call {
                    function: function.to_string(),
                    value: Box::new(value),
                })
            }
            _ => Err(Error {
                error: ErrorTypes::ExpectedIdentifier,
                pos: token.pos.clone(),
            }),
        },
    }
}

pub fn parse(tokens: &[lexer::Token]) -> Result<Value, Error> {
    let mut it = tokens.iter().peekable();
    let records = parse_multiple_records(&mut it, &lexer::TokenKind::EndOfInput)?;
    Ok(Value::Object(records))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_number_value() -> Result<(), Error> {
        let tokens = lexer::tokenize("42");

        let mut it = tokens.iter().peekable();
        let value = parse_value(&mut it)?;
        assert_eq!(value, Value::Number(42));

        Ok(())
    }

    #[test]
    fn parsing_string_value() -> Result<(), Error> {
        let tokens = lexer::tokenize("\"some\"");
        let mut it = tokens.iter().peekable();
        let value = parse_value(&mut it)?;
        assert_eq!(value, Value::String("some".to_string()));
        Ok(())
    }

    #[test]
    fn parsing_float_value() -> Result<(), Error> {
        let tokens = lexer::tokenize("4.20");
        let mut it = tokens.iter().peekable();
        let value = parse_value(&mut it)?;
        assert_eq!(value, Value::Float(4.20));
        Ok(())
    }

    #[test]
    fn parsing_value_call() -> Result<(), Error> {
        let tokens = lexer::tokenize("@call 2");
        let mut it = tokens.iter().peekable();
        let value = parse_value(&mut it)?;
        assert_eq!(
            value,
            Value::Call(Call {
                function: "call".to_string(),
                value: Box::new(Value::Number(2)),
            },)
        );
        Ok(())
    }

    #[test]
    fn test_parse_record() -> Result<(), Error> {
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
