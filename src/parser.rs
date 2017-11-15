use Object;

use std::iter::Peekable;
use std::slice::Iter;
use std::str::Chars;

pub struct Parser<'a> {
    position: usize,
    input: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
}

impl<'a> Parser<'a> {
    pub fn parse(input: &'a str) -> Vec<Token> {
        let input = input.chars().peekable();
        let mut parser = Parser {
            position: 0,
            input: input,
            tokens: Vec::new(),
        };
        parser._parse();

        parser.tokens
    }

    fn next(&mut self) -> Option<char> {
        if let Some(c) = self.input.next() {
            self.position += 1;
            Some(c)
        } else {
            None
        }
    }

    fn _parse(&mut self) {
        while let Some(c) = self.next() {
            match c {
                '(' => self.tokens.push(Token::LeftParen),
                ')' => self.tokens.push(Token::RightParen),
                '\'' => self.tokens.push(Token::Quote),
                '"' => self.parse_string(),
                '#' => self.parse_bool(),
                c if c.is_whitespace() => {}
                '0' ... '9' => self.parse_number(c),
                'A' ... 'Z' | 'a' ... 'z' |
                '!' | '$' | '%' | '&' | '*' | '/' | ':' |
                '<' | '=' | '>' | '?' | '~' | '_' | '^' => self.parse_symbol(c),
                _ => panic!("unexpected input {} at {}", c, self.position),
            }
        }
    }

    pub fn parse_string(&mut self) {
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
                    panic!("unexpected end of input");
                },
                '"' => {
                    self.tokens.push(Token::String(buf));
                    return;
                }
                _ => buf.push(c),
            }
        }
        panic!("unexpected end of input");
    }

    pub fn parse_bool(&mut self) {
        match self.next() {
            Some('t') => self.tokens.push(Token::Bool(true)),
            Some('f') => self.tokens.push(Token::Bool(false)),
            Some(_) => panic!("unexpected input"),
            _ => panic!("unexpected end of input"),
        }

        match self.next() {
            Some(c) if c.is_whitespace() => {}
            Some('(') => self.tokens.push(Token::LeftParen),
            Some(')') => self.tokens.push(Token::RightParen),
            _ => panic!("unexpected input"),
        }
    }

    pub fn parse_number(&mut self, first: char) {
        let mut buf = String::new();
        buf.push(first);
        while let Some(c) = self.next() {
            match c {
                c if c.is_whitespace() => {
                    self.tokens.push(Token::Number(buf));
                    return;
                }
                '0' ... '9' => buf.push(c),
                '(' => {
                    self.tokens.push(Token::Number(buf));
                    self.tokens.push(Token::LeftParen);
                    return;
                }
                ')' => {
                    self.tokens.push(Token::Number(buf));
                    self.tokens.push(Token::RightParen);
                    return;
                }
                _ => panic!("unexpected input"),
            }
        }
    }

    pub fn parse_symbol(&mut self, first: char) {
        let mut buf = String::new();
        buf.push(first);
        while let Some(c) = self.next() {
            match c {
                'A' ... 'Z' | 'a' ... 'z' | '0' ... '9' |
                '!' | '$' | '%' | '&' | '*' | '/' | ':' |
                '<' | '=' | '>' | '?' | '~' | '_' | '^' => buf.push(c),
                c if c.is_whitespace() => {
                    if buf == "nil" {
                        return self.tokens.push(Token::Nil);
                    } else {
                        return self.tokens.push(Token::Symbol(buf));
                    }
                }
                ')' => {
                    if buf == "nil" {
                        self.tokens.push(Token::Nil);
                    } else {
                        self.tokens.push(Token::Symbol(buf));
                    }
                    return self.tokens.push(Token::RightParen);
                }
                _ => panic!("unexpected input"),
            }
        }
    }
}

#[derive(Debug)]
pub enum Token {
    LeftParen,
    RightParen,
    Quote,
    Nil,
    Bool(bool),
    String(String),
    Number(String),
    Symbol(String),
}

impl Token {
    pub fn build_ast(tokens: Vec<Self>) -> Vec<Object> {
        use self::Token::*;
        let mut exprs = Vec::new();
        let mut tokens = tokens.iter();
        while let Some(token) = tokens.next() {
            match token {
                LeftParen => {
                    let mut list = Object::Nil;
                    Self::parse_expr(&mut tokens, &mut list);
                    exprs.push(list);
                }
                RightParen => panic!("unexpected right paren"),
                Quote => {
                    let list = Self::parse_quote(&mut tokens);
                    exprs.push(list);
                }
                Nil => exprs.push(Object::Nil),
                Bool(b) => exprs.push(Object::Bool(*b)),
                Number(i) => exprs.push(Object::Number(i.parse::<i64>().unwrap())),
                String(s) => exprs.push(Object::String(s.to_owned())),
                Symbol(s) => exprs.push(Object::Symbol(s.to_owned())),
            }
        }

        exprs
    }

    fn parse_quote<'a>(tokens: &mut Iter<'a, Self>) -> Object {
        use self::Token::*;
        let quoted = match *tokens.next().unwrap() {
            Symbol(ref s) => Object::Symbol(s.to_owned()),
            Number(ref i) => {
                return Object::Number(i.parse::<i64>().unwrap());
            },
            String(ref s) => {
                return Object::String(s.to_owned());
            },
            LeftParen => {
                let mut list = Object::Nil;
                Self::parse_expr(tokens, &mut list);
                list
            },
            _ => panic!("unexpected token in quote"),
        };
        Object::cons(Object::Symbol("quote".to_string()),
                     Object::cons(quoted, Object::Nil))
    }

    fn parse_expr<'a>(tokens: &mut Iter<'a, Self>, list: &mut Object) {
        use self::Token::*;
        let mut parens = 1;
        while let Some(token) = tokens.next() {
            match token {
                LeftParen => {
                    let mut l = Object::Nil;
                    Self::parse_expr(tokens, &mut l);
                    *list = list.push(l);
                },
                RightParen => {
                    parens -= 1;
                    break;
                }
                Quote => {
                    let l = Self::parse_quote(tokens);
                    *list = list.push(l);
                },
                Nil => *list = list.push(Object::Nil),
                Bool(b) => *list = list.push(Object::Bool(*b)),
                String(s) => *list = list.push(Object::String(s.to_owned())),
                Symbol(s) => *list = list.push(Object::Symbol(s.to_owned())),
                Number(i) => *list = list.push(Object::Number(i.parse::<i64>().unwrap())),
            }
        }
        assert!(parens == 0);
    }
}
