use super::ParseError;
use Object;

use std::iter::Peekable;
use std::mem;
use std::slice::Iter;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, is_enum_variant)]
pub enum Token {
    Comment(String),
    BlockComment(String),
    LeftParen,
    RightParen,
    Dot,
    Quote,
    Nil,
    Bool(bool),
    String(String),
    Integer(String),
    Rational(String),
    Real(String),
    ComplexInt(Option<String>, Option<String>),
    ComplexRat(Option<String>, Option<String>),
    ComplexReal(Option<String>, Option<String>),
    Symbol(String),
}

impl Token {
    fn is_number(&self) -> bool {
        match self {
            Token::Integer(_) | Token::Rational(_) |
            Token::Real(_) | Token::ComplexInt(_, _) |
            Token::ComplexRat(_, _) | Token::ComplexReal(_, _) => true,
            _ => false,
        }
    }

    fn to_object(&self) -> Object {
        match self {
            Token::Nil => Object::Nil,
            Token::Bool(b) => Object::Bool(*b),
            Token::String(s) => Object::String(s.to_owned()),
            num if self.is_number() => Object::Number(::Number::from_token(num)),
            Token::Symbol(s) => Object::Symbol(s.to_owned()),
            _ => unreachable!(),
        }
    }

    pub fn build_ast(tokens: Vec<Self>) -> Result<Vec<Object>, ParseError> {
        use self::Token::*;
        let mut exprs = Vec::new();
        let mut tokens = tokens.iter().peekable();
        while let Some(token) = tokens.next() {
            match token {
                LeftParen => {
                    let list = Self::parse_expr(&mut tokens)?;
                    exprs.push(list);
                }
                RightParen => return Err(ParseError::UnexpectedCloseParen),
                Dot => return Err(ParseError::IllegalUse),
                Quote => {
                    let list = Self::parse_quote(&mut tokens)?;
                    exprs.push(list);
                }
                _ => exprs.push(token.to_object()),
            }
        }

        Ok(exprs)
    }

    fn parse_quote<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Object, ParseError> {
        use self::Token::*;
        let next = if let Some(t) = tokens.next() {
            t
        } else {
            return Err(ParseError::BadQuote);
        };

        let quoted = match next {
            Symbol(s) => Object::Symbol(s.to_owned()),
            LeftParen => {
                Self::parse_expr(tokens)?
            },
            Dot => return Err(ParseError::IllegalUse),
            RightParen => return Err(ParseError::UnexpectedCloseParen),
            // TODO: what should happen on `''a`?
            Quote => Self::parse_quote(tokens)?,
            _ => return Ok(next.to_object()),
        };
        Ok(Object::cons(Object::Symbol("quote".to_string()),
                        Object::cons(quoted, Object::Nil)))
    }

    fn parse_expr<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Object, ParseError> {
        use self::Token::*;
        let mut parens = 1;
        let mut stack = Vec::new();
        let mut list = Object::Nil;

        while let Some(token) = tokens.next() {
            match token {
                LeftParen => {
                    parens += 1;
                    let mut new = Object::Nil;
                    mem::swap(&mut new, &mut list);
                    stack.push(new);
                }
                RightParen => {
                    parens -= 1;
                    if parens == 0 {
                        debug_assert!(stack.is_empty());
                        return Ok(list);
                    }
                    let mut old = stack.pop().unwrap();
                    mem::swap(&mut list, &mut old);
                    list = list.push(old);
                }
                Dot => {
                    let token = match tokens.next() {
                        Some(t) => t,
                        None => return Err(ParseError::EOF),
                    };

                    match token {
                        RightParen => return Err(ParseError::UnexpectedCloseParen),
                        Dot => return Err(ParseError::IllegalUse),
                        LeftParen => {
                            let l = Token::parse_expr(tokens)?;
                            list.set_cdr(l);
                        }
                        _ => {
                            if tokens.peek() != Some(&&Token::RightParen)  || list.is_null() {
                                return Err(ParseError::IllegalUse);
                            }
                            list.set_cdr(token.to_object());
                        }
                    }
                }
                Quote => {
                    let l = Self::parse_quote(tokens)?;
                    list = list.push(l);
                },
                Nil => list = list.push(Object::Nil),
                Bool(b) => list = list.push(Object::Bool(*b)),
                String(s) => list = list.push(Object::String(s.to_owned())),
                Symbol(s) => list = list.push(Object::Symbol(s.to_owned())),
                num if token.is_number() =>
                    list = list.push(Object::Number(::Number::from_token(num))),
                _ => unreachable!(),
            }
        }

        Err(ParseError::UnbalancedParen)
    }
}
