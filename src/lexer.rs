use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    // Value carrying
    ID(String),
    Number(i32),
    String(String),

    // Symbols
    Separator,
    Assign,
    LeftBrace,
    RightBrace,

    Dot,

    ValueCall,

    // Control
    EndOfInput,
}

fn skip_spaces(it: &mut Peekable<Chars>) {
    while let Some(ch) = it.peek() {
        if ch.is_whitespace() && *ch != '\n' {
            it.next();
        } else {
            break;
        }
    }
}

fn lex_number(it: &mut Peekable<Chars>) -> i32 {
    let mut x = 0;

    while let Some(ch) = it.peek() {
        if let Some(digit) = ch.to_digit(10) {
            x *= 10;
            x += digit as i32;
            it.next();
        } else {
            break;
        }
    }

    x
}

fn lex_ident(it: &mut Peekable<Chars>) -> String {
    let mut x = String::new();

    while let Some(ch) = it.peek() {
        if ch.is_alphanumeric() {
            x.push(*ch);
            it.next();
        } else {
            break;
        }
    }

    x
}

fn lex_string(it: &mut Peekable<Chars>) -> String {
    let mut x = String::new();

    if let Some(ch) = it.peek() {
        if *ch == '"' {
            it.next();
            while let Some(ch) = it.peek() {
                if *ch == '"' {
                    it.next();
                    break;
                }
                x.push(*ch);
                it.next();
            }
        }
    }

    x
}

pub fn tokenize(s: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut it = s.chars().peekable();
    loop {
        skip_spaces(&mut it);
        let ch;
        if let Some(x) = it.peek() {
            ch = *x;
        } else {
            tokens.push(Token::EndOfInput);
            break;
        }

        match ch {
            ';' | '\n' => {
                it.next();
                tokens.push(Token::Separator);
                continue;
            }
            '=' => {
                it.next();
                tokens.push(Token::Assign);
                continue;
            }

            '{' => {
                it.next();
                tokens.push(Token::LeftBrace);
            }
            '}' => {
                it.next();
                tokens.push(Token::RightBrace);
            }

            '"' => {
                tokens.push(Token::String(lex_string(&mut it)));
                continue;
            }
            '.' => {
                it.next();
                tokens.push(Token::Dot);
            }

            '@' => {
                it.next();
                tokens.push(Token::ValueCall);
            }

            _ => {
                if ch.is_ascii_digit() {
                    tokens.push(Token::Number(lex_number(&mut it)));
                    continue;
                } else if ch.is_alphanumeric() {
                    tokens.push(Token::ID(lex_ident(&mut it)));
                    continue;
                }

                unimplemented!("lexer doesn't know how to handle: {}", ch);
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skipping_spaces() {
        let mut it = " \t \tword".chars().peekable();
        skip_spaces(&mut it);
        assert_eq!(it.collect::<String>(), "word");
    }

    #[test]
    fn lexing_number() {
        let mut it = "2023".chars().peekable();
        assert_eq!(lex_number(&mut it), 2023);
    }

    #[test]
    fn lexing_string() {
        let mut it = "\"some\"".chars().peekable();
        assert_eq!(lex_string(&mut it), "some");
    }

    #[test]
    fn tokenize_braces() {
        assert_eq!(
            tokenize("{}"),
            [Token::LeftBrace, Token::RightBrace, Token::EndOfInput,],
        );
    }

    #[test]
    fn tokenize_value_call() {
        assert_eq!(tokenize("@"), [Token::ValueCall, Token::EndOfInput,],);
    }

    #[test]
    fn tokenize_record() {
        let data = "x = 2";

        let tokens = tokenize(data);

        assert_eq!(
            tokens,
            [
                Token::ID("x".to_owned()),
                Token::Assign,
                Token::Number(2),
                Token::EndOfInput,
            ],
        );
    }

    #[test]
    fn tokenize_multiple_records() {
        let data = "x = 2\ny = 3;z=4";

        let tokens = tokenize(data);

        assert_eq!(
            tokens,
            [
                Token::ID("x".to_owned()),
                Token::Assign,
                Token::Number(2),
                Token::Separator,
                Token::ID("y".to_string()),
                Token::Assign,
                Token::Number(3),
                Token::Separator,
                Token::ID("z".to_string()),
                Token::Assign,
                Token::Number(4),
                Token::EndOfInput,
            ],
        );
    }
}
