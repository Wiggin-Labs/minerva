use super::{ParseError, Token};

use regex::Regex;

use std::iter::Peekable;
use std::str::Chars;

type ParseResult = Result<(), ParseError>;

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

    fn peek(&mut self) -> Option<char> {
        match self.input.peek() {
            Some(c) => Some(*c),
            None => None,
        }
    }

    fn _parse(&mut self) -> ParseResult {
        while let Some(c) = self.next() {
            match c {
                c if is_pair_start(c) => self.tokens.push(Token::LeftParen),
                c if is_pair_end(c) => self.tokens.push(Token::RightParen),
                '\'' => self.tokens.push(Token::Quote),
                '"' => self.parse_string()?,
                '|' => self.parse_identifier(String::new(), true)?,
                ';' => self.parse_comment(c)?,
                '#' => {
                    match self.peek() {
                        Some('|') => self.parse_block_comment()?,
                        _ => {},
                    }
                    //self.parse_bool()?,
                }
                c if c.is_whitespace() => {}
                '.' => match self.peek() {
                    Some(c) => match c {
                        c if is_delimiter(c) => self.tokens.push(Token::Dot),
                        c => self.parse_ambiguous('.')?,
                    },
                    None => self.tokens.push(Token::Dot),
                },
                '0' ... '9' | '+' | '-' => self.parse_ambiguous(c)?,
                _ => {
                    let mut buf = String::new();
                    buf.push(c);
                    self.parse_identifier(buf, false)?;
                }
            }
        }
        Ok(())
    }

    fn parse_ambiguous(&mut self, c: char) -> ParseResult {
        let mut buf = String::new();
        buf.push(c);

        while let Some(c) = self.next() {
            match c {
                '0' ... '9' | '+' | '-' | '/' | '.' | 'e' | 'i' => buf.push(c),
                c if is_pair_start(c) => {
                    self.distinguish_ambiguous(buf)?;
                    self.tokens.push(Token::LeftParen);
                    return Ok(());
                }
                c if is_pair_end(c) => {
                    self.distinguish_ambiguous(buf)?;
                    self.tokens.push(Token::RightParen);
                    return Ok(());
                }
                c if c.is_whitespace() => break,
                '\\' => match self.next() {
                    Some(c) => {
                        buf.push(c);
                        return self.parse_identifier(buf, false);
                    }
                    None => return Err(ParseError::EOF),
                },
                _ => {
                    buf.push(c);
                    return self.parse_identifier(buf, c == '|');
                }
            }
        }
        self.distinguish_ambiguous(buf)
    }

    fn distinguish_ambiguous(&mut self, buf: String) -> ParseResult {
        const _INT: &'static str = r"\d+";
        const _RAT: &'static str = r"\d+(?:/\d+)?";
        const _REAL: &'static str = r"\d*\.?\d+(?:[eE][-+]?\d+)?";
        lazy_static! {
            static ref INTEGER: Regex = Regex::new(&format!("^[+-]?{}$", _INT)).expect("1");
            static ref RATIONAL: Regex = Regex::new(&format!("^[+-]?{}$", _RAT)).expect("2");
            static ref REAL: Regex = Regex::new(&format!("^[+-]?{}$", _REAL)).unwrap();
            static ref COMPLEX_INT: Regex = Regex::new(&format!("^[+-]?{}[+-]{0}i$", _INT)).unwrap();
            static ref COMPLEX_RAT: Regex = Regex::new(&format!("^[+-]?{}[+-]{0}i$", _RAT)).unwrap();
            static ref COMPLEX_REAL: Regex = Regex::new(&format!("^[+-]?(?:{}|{})[+-](?:{0}|{1})i$", _REAL, _RAT)).unwrap();
        }

        if INTEGER.is_match(&buf) {
            self.tokens.push(Token::Integer(buf));
        } else if RATIONAL.is_match(&buf) {
            self.tokens.push(Token::Rational(buf));
        } else if REAL.is_match(&buf) {
            self.tokens.push(Token::Real(buf));
        } else if COMPLEX_INT.is_match(&buf) {
            self.tokens.push(Token::ComplexInt(buf));
        } else if COMPLEX_RAT.is_match(&buf) {
            self.tokens.push(Token::ComplexRat(buf));
        } else if COMPLEX_REAL.is_match(&buf) {
            self.tokens.push(Token::ComplexReal(buf));
        } else {
            self.tokens.push(Token::Symbol(buf));
        }
        Ok(())
    }

    fn parse_identifier(&mut self, mut buf: String, mut in_bar: bool) -> ParseResult {
        while let Some(c) = self.next() {
            match c {
                '\\' => match self.next() {
                    Some(c) => buf.push(c),
                    None => return Err(ParseError::EOF),
                },
                '|' => in_bar = !in_bar,
                c if is_delimiter(c) => if in_bar {
                    buf.push(c);
                } else {
                    self.tokens.push(Token::Symbol(buf));
                    return match c {
                        c if c.is_whitespace() => Ok(()),
                        c if is_pair_start(c) => Ok(self.tokens.push(Token::LeftParen)),
                        c if is_pair_end(c) => Ok(self.tokens.push(Token::RightParen)),
                        '"' => self.parse_string(),
                        ';' => self.parse_comment(c),
                        _ => panic!("Parser error"),
                    };
                },
                _ => buf.push(c),
            }
        }
        self.tokens.push(Token::Symbol(buf));
        Ok(())
    }

    pub fn parse_string(&mut self) -> ParseResult {
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

    /*
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
    */

    fn parse_block_comment(&mut self) -> ParseResult {
        let mut buf = String::from("#|");
        let mut nesting = 1;
        while let Some(c) = self.next() {
            match c {
                '|' => match self.next() {
                    Some('#') => {
                        nesting -= 1;
                        buf.push('|');
                        buf.push('#');
                        if nesting == 0 {
                            self.tokens.push(Token::BlockComment(buf));
                            return Ok(());
                        }
                    }
                    Some(c) => buf.push(c),
                    None => return Err(ParseError::EOF),
                },
                '#' => match self.next() {
                    Some('|') => {
                        nesting += 1;
                        buf.push('#');
                        buf.push('|');
                    }
                    Some(c) => buf.push(c),
                    None => return Err(ParseError::EOF),
                },
                _ => buf.push(c),
            }
        }
        Err(ParseError::EOF)
    }

    // TODO
    fn parse_comment(&mut self, c: char) -> ParseResult {
        let mut buf = String::from(";");

        while let Some(c) = self.next() {
            match c {
                '\\' => match self.next() {
                    Some(c) => {
                        buf.push('\\');
                        buf.push(c);
                    }
                    None => break,
                },
                '\n' => break,
                _ => buf.push(c),
            }
        }
        self.tokens.push(Token::Comment(buf));
        Ok(())
    }
}

fn is_delimiter(c: char) -> bool {
    match c {
        c if is_pair_start(c) => true,
        c if is_pair_end(c) => true,
        c if c.is_whitespace() => true,
        '"' | ';' => true,
        _ => false,
    }
}

fn is_pair_start(c: char) -> bool {
    match c {
        '(' | '[' | '{' => true,
        _ => false,
    }
}

fn is_pair_end(c: char) -> bool {
    match c {
        ')' | ']' | '}' => true,
        _ => false,
    }
}
