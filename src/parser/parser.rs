use super::{ParseError, Token};

use std::iter::Peekable;
use std::str::Chars;

pub struct Parser<'a> {
    position: usize,
    input: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
}

impl<'a> Parser<'a> {
    pub fn parse(input: &'a str) -> Result<Vec<Token>, ParseError> {
        let input = input.chars().peekable();
        let mut parser = Parser {
            position: 0,
            input: input,
            tokens: Vec::new(),
        };
        parser._parse()?;

        Ok(parser.tokens)
    }

    fn next(&mut self) -> Option<char> {
        if let Some(c) = self.input.next() {
            self.position += 1;
            Some(c)
        } else {
            None
        }
    }

    fn _parse(&mut self) -> Result<(), ParseError> {
        while let Some(c) = self.next() {
            match c {
                '(' => self.tokens.push(Token::LeftParen),
                ')' => self.tokens.push(Token::RightParen),
                '.' => self.tokens.push(Token::Dot),
                '\'' => self.tokens.push(Token::Quote),
                '"' => self.parse_string()?,
                '#' => self.parse_bool()?,
                c if c.is_whitespace() => {}
                '0' ... '9' => self.parse_number(c)?,
                c if is_symbol_char(c, true) => self.parse_symbol(c)?,
                _ => return Err(ParseError::Input),
            }
        }
        Ok(())
    }

    pub fn parse_string(&mut self) -> Result<(), ParseError> {
        let mut buf = String::new();
        while let Some(c) = self.next() {
            match c {
                '\\' => if let Some(c) = self.next() {
                    match c {
                        'n' => buf.push('\n'),
                        't' => buf.push('\t'),
                        // TODO: handle other escapes
                        _ => buf.push(c),
                    }
                } else {
                    return Err(ParseError::EOF);
                },
                '"' => {
                    self.tokens.push(Token::String(buf));
                    return Ok(());
                }
                _ => buf.push(c),
            }
        }
        Err(ParseError::EOF)
    }

    pub fn parse_bool(&mut self) -> Result<(), ParseError> {
        match self.next() {
            Some('t') => self.tokens.push(Token::Bool(true)),
            Some('f') => self.tokens.push(Token::Bool(false)),
            Some(_) => return Err(ParseError::Input),
            _ => return Err(ParseError::EOF),
        }

        match self.next() {
            Some(c) if c.is_whitespace() => {},
            Some('(') => self.tokens.push(Token::LeftParen),
            Some(')') => self.tokens.push(Token::RightParen),
            Some(_) => return Err(ParseError::Input),
            None => {},
        }
        Ok(())
    }

    pub fn parse_number(&mut self, first: char) -> Result<(), ParseError> {
        let mut buf = String::new();
        buf.push(first);
        while let Some(c) = self.next() {
            match c {
                c if c.is_whitespace() => {
                    self.tokens.push(Token::Number(buf));
                    return Ok(());
                }
                '0' ... '9' => buf.push(c),
                '(' => {
                    self.tokens.push(Token::Number(buf));
                    self.tokens.push(Token::LeftParen);
                    return Ok(());
                }
                ')' => {
                    self.tokens.push(Token::Number(buf));
                    self.tokens.push(Token::RightParen);
                    return Ok(());
                }
                _ => return Err(ParseError::Input),
            }
        }
        self.tokens.push(Token::Number(buf));
        Ok(())
    }

    pub fn parse_symbol(&mut self, first: char) -> Result<(), ParseError> {
        let mut buf = String::new();
        buf.push(first);
        while let Some(c) = self.next() {
            match c {
                c if is_symbol_char(c, false) => buf.push(c),
                c if c.is_whitespace() => {
                    if buf == "nil" {
                        self.tokens.push(Token::Nil);
                        return Ok(());
                    } else {
                        self.tokens.push(Token::Symbol(buf));
                        return Ok(());
                    }
                }
                ')' => {
                    if buf == "nil" {
                        self.tokens.push(Token::Nil);
                    } else {
                        self.tokens.push(Token::Symbol(buf));
                    }
                    self.tokens.push(Token::RightParen);
                    return Ok(())
                }
                _ => return Err(ParseError::Input),
            }
        }
        self.tokens.push(Token::Symbol(buf));
        Ok(())
    }
}

fn is_symbol_char(c: char, start: bool) -> bool {
    match c {
        'a' ... 'z' | 'A' ... 'Z' | '-' | '+' |
        '!' | '$' | '%' | '&' | '*' | '/' | ':' |
        '<' | '=' | '>' | '?' | '~' | '_' | '^' => true,
        '0' ... '9' => !start,
        _ => false,
    }
}
