use crate::ast::{Record, Value};
use crate::lexer;
use std::fmt;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum ParseError {
    EndOfInput,
    ExpectedIdentifier,
    ExpectedValue,
    ExpectedAssign,
    ExpectedNumber,
}

impl fmt::Display for ParseError {
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

fn parse_value<'a, T>(it: &mut Peekable<T>) -> Result<Value, ParseError>
where
    T: Iterator<Item = &'a lexer::Token>,
{
    match it.peek() {
        None => Err(ParseError::EndOfInput),
        Some(token) => match token {
            lexer::Token::Number(num) => {
                it.next();
                if let Some(token) = it.peek() {
                    if **token == lexer::Token::Dot {
                        it.next();
                        if let Some(token) = it.peek() {
                            println!("got {:?}", token);
                            match token {
                                lexer::Token::Number(n) => {
                                    // see https://stackoverflow.com/a/69298721
                                    let decimals = n.checked_ilog10().unwrap_or(0) + 1;
                                    let x: f32 = *num as f32
                                        + 10.0_f32.powi(-(decimals as i32)) * (*n as f32);
                                    return Ok(Value::Float(x));
                                }
                                _ => {
                                    return Err(ParseError::ExpectedNumber);
                                }
                            }
                        } else {
                            return Err(ParseError::EndOfInput);
                        }
                    }
                }
                Ok(Value::Number(*num))
            }
            lexer::Token::String(s) => Ok(Value::String(s.to_owned())),
            lexer::Token::LeftBrace => {
                it.next();
                let records = parse_multiple_records(it, lexer::Token::RightBrace)?;
                Ok(Value::Object(records))
            }
            _ => Err(ParseError::ExpectedValue),
        },
    }
}

fn parse_record<'a, T>(it: &mut Peekable<T>) -> Result<Record, ParseError>
where
    T: Iterator<Item = &'a lexer::Token>,
{
    match it.peek() {
        None => Err(ParseError::EndOfInput),
        Some(token) => match token {
            lexer::Token::ID(id) => {
                it.next();

                if let Some(token) = it.peek() {
                    if **token == lexer::Token::Assign {
                        it.next();

                        let value = parse_value(it)?;

                        Ok(Record {
                            id: id.to_owned(),
                            value,
                        })
                    } else {
                        Err(ParseError::ExpectedAssign)
                    }
                } else {
                    Err(ParseError::EndOfInput)
                }
            }
            _ => Err(ParseError::ExpectedIdentifier),
        },
    }
}

fn parse_multiple_records<'a, T>(
    it: &mut Peekable<T>,
    end: lexer::Token,
) -> Result<Vec<Record>, ParseError>
where
    T: Iterator<Item = &'a lexer::Token>,
{
    let mut records: Vec<Record> = Vec::new();
    loop {
        match it.peek() {
            None => return Err(ParseError::EndOfInput),
            Some(token) => match token {
                lexer::Token::ID(_) => {
                    let record = parse_record(it)?;
                    records.push(record);
                }
                lexer::Token::Separator => (),
                _ if end == **token => break,
                _ => todo!("{:?}", token),
            },
        };

        it.next();
    }

    Ok(records)
}

pub fn parse(tokens: &[lexer::Token]) -> Result<Vec<Record>, ParseError> {
    let mut it = tokens.iter().peekable();
    parse_multiple_records(&mut it, lexer::Token::EndOfInput)
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
    fn parsing_string_value() -> Result<(), ParseError> {
        let tokens = lexer::tokenize("\"some\"");
        let mut it = tokens.iter().peekable();
        let value = parse_value(&mut it)?;
        assert_eq!(value, Value::String("some".to_string()));
        Ok(())
    }

    #[test]
    fn parsing_float_value() -> Result<(), ParseError> {
        let tokens = lexer::tokenize("4.20");
        let mut it = tokens.iter().peekable();
        let value = parse_value(&mut it)?;
        assert_eq!(value, Value::Float(4.20));
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
