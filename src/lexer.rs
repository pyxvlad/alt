
use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
pub enum Token {
    ID(String),
    Assign,
    Number(i32),
    Separator,
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

    return x;
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

    return x;
}

fn lex_string(it: &mut Peekable<Chars>) -> String {
    let mut x = String::new();

    return x;
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
            break;
        }
        if ch == ';' || ch == '\n' {
            it.next();
            tokens.push(Token::Separator);
            continue;
        }
        if ch == '=' {
            it.next();
            tokens.push(Token::Assign);
            continue;
        }

        if ch.is_digit(10) {
            tokens.push(Token::Number(lex_number(&mut it)));
            continue;
        }

        if ch.is_alphanumeric() {
            tokens.push(Token::ID(lex_ident(&mut it)));
            continue;
        }

        unimplemented!("lexer doesn't know how to handle: {}", ch);
    }

    return tokens;
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
    fn tokenize_record() {
        let data = "x = 2";

        let tokens = tokenize(data);

        assert_eq!(
            tokens,
            [Token::ID("x".to_owned()), Token::Assign, Token::Number(2),],
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
                Token::Number(4)
            ],
        );
    }
}

