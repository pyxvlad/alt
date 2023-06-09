use core::fmt;
use std::{iter::Peekable, str::CharIndices};

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub enum TokenKind {
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
    RecordCall,

    // Control
    #[default]
    EndOfInput,
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct FilePos {
    pub start: usize,
    pub end: usize,
}

impl fmt::Display for FilePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} chars)", self.start, self.end - self.start)
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: FilePos,
}

impl Token {}

fn skip_spaces(it: &mut Peekable<CharIndices>) {
    while let Some((_, ch)) = it.peek() {
        if ch.is_whitespace() && *ch != '\n' {
            it.next();
        } else {
            break;
        }
    }
}

fn lex_number(it: &mut Peekable<CharIndices>) -> Token {
    let mut token: Token = Default::default();

    let mut x = 0;
    let mut first = true;
    loop {
        match it.peek() {
            None => {
                token.kind = TokenKind::Number(x);
                token.pos.end = token.pos.start + x.ilog10() as usize + 1;
                break;
            }
            Some((pos, ch)) => {
                if first {
                    token.pos.start = *pos;
                    first = false;
                }
                if let Some(digit) = ch.to_digit(10) {
                    x *= 10;
                    x += digit as i32;
                    it.next();
                } else {
                    token.kind = TokenKind::Number(x);
                    token.pos.end = *pos;
                    break;
                }
            }
        }
    }

    token
}

fn lex_ident(it: &mut Peekable<CharIndices>) -> Token {
    let mut token: Token = Default::default();
    let mut x = String::new();

    let mut first = true;
    loop {
        match it.peek() {
            None => {
                token.pos.end = token.pos.start + x.len();
                token.kind = TokenKind::ID(x);
                break;
            }
            Some((pos, ch)) => {
                if first {
                    first = false;
                    token.pos.start = *pos;
                }
                if ch.is_alphanumeric() || *ch == '-' || *ch == '_' {
                    x.push(*ch);
                    it.next();
                } else {
                    token.kind = TokenKind::ID(x);
                    token.pos.end = *pos;
                    break;
                }
            }
        }
    }

    token
}

fn lex_string(it: &mut Peekable<CharIndices>) -> Token {
    let mut token: Token = Default::default();
    let mut x = String::new();

    if let Some((pos, ch)) = it.peek() {
        if *ch == '"' {
            token.pos.start = *pos;
            it.next();
            while let Some((pos, ch)) = it.peek() {
                if *ch == '"' {
                    token.pos.end = *pos;
                    token.kind = TokenKind::String(x);
                    it.next();
                    break;
                }
                x.push(*ch);
                it.next();
            }
        }
    }

    token
}

pub fn tokenize(s: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut it = s.char_indices().peekable();
    loop {
        skip_spaces(&mut it);
        let ch;
        let pos;
        if let Some(x) = it.peek() {
            pos = x.0;
            ch = x.1;
        } else {
            pos = s.len();
            tokens.push(Token {
                pos: FilePos {
                    start: pos,
                    end: pos,
                },
                kind: TokenKind::EndOfInput,
            });
            break;
        }

        match ch {
            ';' | '\n' => {
                it.next();
                tokens.push(Token {
                    pos: FilePos {
                        start: pos,
                        end: pos + 1,
                    },
                    kind: TokenKind::Separator,
                });
                continue;
            }
            '=' => {
                it.next();
                tokens.push(Token {
                    pos: FilePos {
                        start: pos,
                        end: pos + 1,
                    },
                    kind: TokenKind::Assign,
                });
                continue;
            }

            '{' => {
                it.next();
                tokens.push(Token {
                    pos: FilePos {
                        start: pos,
                        end: pos + 1,
                    },
                    kind: TokenKind::LeftBrace,
                });
                continue;
            }
            '}' => {
                it.next();
                tokens.push(Token {
                    pos: FilePos {
                        start: pos,
                        end: pos + 1,
                    },
                    kind: TokenKind::RightBrace,
                });
                continue;
            }

            '"' => {
                tokens.push(lex_string(&mut it));
                continue;
            }
            '.' => {
                it.next();
                tokens.push(Token {
                    pos: FilePos {
                        start: pos,
                        end: pos + 1,
                    },
                    kind: TokenKind::Dot,
                });
                continue;
            }

            '@' => {
                it.next();
                tokens.push(Token {
                    pos: FilePos {
                        start: pos,
                        end: pos + 1,
                    },
                    kind: TokenKind::ValueCall,
                });
                continue;
            }

            '#' => {
                it.next();
                tokens.push(Token {
                    pos: FilePos {
                        start: pos,
                        end: pos + 1,
                    },
                    kind: TokenKind::RecordCall,
                });
                continue;
            }

            _ if ch.is_ascii_digit() => {
                tokens.push(lex_number(&mut it));
                continue;
            }
            _ if ch.is_alphanumeric() => {
                tokens.push(lex_ident(&mut it));
                continue;
            }

            _ => unimplemented!("lexer doesn't know how to handle: {}", ch),
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skipping_spaces() {
        let mut it = " \t \tword".char_indices().peekable();
        skip_spaces(&mut it);
        assert_eq!(it.map(|x| x.1).collect::<String>(), "word");
    }

    #[test]
    fn lexing_number() {
        let mut it = "2023".char_indices().peekable();
        assert_eq!(lex_number(&mut it).kind, TokenKind::Number(2023));
    }

    #[test]
    fn x() {
        "2023"
            .char_indices()
            .peekable()
            .for_each(|a| println!("@{} {:?}", a.0, a.1));
    }

    #[test]
    fn lexing_string() {
        let mut it = "\"some\"".char_indices().peekable();
        assert_eq!(
            lex_string(&mut it).kind,
            TokenKind::String("some".to_string())
        );
    }

    #[test]
    fn tokenize_braces() {
        assert_eq!(
            tokenize("{}")
                .iter()
                .map(|t| t.kind.clone())
                .collect::<Vec<TokenKind>>(),
            [
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::EndOfInput,
            ],
        );
    }

    #[test]
    fn tokenize_value_call() {
        assert_eq!(
            tokenize("@")
                .iter()
                .map(|t| t.kind.clone())
                .collect::<Vec<TokenKind>>(),
            [TokenKind::ValueCall, TokenKind::EndOfInput,],
        );
    }

    #[test]
    fn tokenize_record_call() {
        assert_eq!(
            tokenize("#")
                .iter()
                .map(|t| t.kind.clone())
                .collect::<Vec<TokenKind>>(),
            [TokenKind::RecordCall, TokenKind::EndOfInput,],
        );
    }

    #[test]
    fn tokenize_record() {
        let data = "x = 2";

        assert_eq!(
            tokenize(data)
                .iter()
                .map(|t| t.kind.clone())
                .collect::<Vec<TokenKind>>(),
            [
                TokenKind::ID("x".to_owned()),
                TokenKind::Assign,
                TokenKind::Number(2),
                TokenKind::EndOfInput,
            ],
        );
    }

    #[test]
    fn tokenize_multiple_records() {
        let data = "x = 2\ny = 3;z=4";

        assert_eq!(
            tokenize(data)
                .iter()
                .map(|t| t.kind.clone())
                .collect::<Vec<TokenKind>>(),
            [
                TokenKind::ID("x".to_owned()),
                TokenKind::Assign,
                TokenKind::Number(2),
                TokenKind::Separator,
                TokenKind::ID("y".to_string()),
                TokenKind::Assign,
                TokenKind::Number(3),
                TokenKind::Separator,
                TokenKind::ID("z".to_string()),
                TokenKind::Assign,
                TokenKind::Number(4),
                TokenKind::EndOfInput,
            ],
        );
    }
}
