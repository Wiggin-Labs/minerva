use super::ParseError;
use Object;

use std::mem;
use std::slice::Iter;

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
    fn is_number(&self) -> bool {
        match self {
            Token::Number(_) => true,
            _ => false,
        }
    }

    pub fn build_ast(tokens: Vec<Self>) -> Result<Vec<Object>, ParseError> {
        use self::Token::*;
        let mut exprs = Vec::new();
        let mut tokens = tokens.iter();
        while let Some(token) = tokens.next() {
            match token {
                LeftParen => {
                    let list = Self::parse_expr(&mut tokens)?;
                    exprs.push(list);
                }
                RightParen => return Err(ParseError::UnexpectedCloseParen),
                Quote => {
                    let list = Self::parse_quote(&mut tokens)?;
                    exprs.push(list);
                }
                Nil => exprs.push(Object::Nil),
                Bool(b) => exprs.push(Object::Bool(*b)),
                num if token.is_number() => exprs.push(Object::Number(::Number::from_token(num))),
                String(s) => exprs.push(Object::String(s.to_owned())),
                Symbol(s) => exprs.push(Object::Symbol(s.to_owned())),
                _ => unreachable!(),
            }
        }

        Ok(exprs)
    }

    fn parse_quote<'a>(tokens: &mut Iter<'a, Self>) -> Result<Object, ParseError> {
        use self::Token::*;
        let next = if let Some(t) = tokens.next() {
            t
        } else {
            return Err(ParseError::BadQuote);
        };

        let quoted = match next {
            Symbol(s) => Object::Symbol(s.to_owned()),
            num if next.is_number() => return Ok(Object::Number(::Number::from_token(num))),
            String(s) => return Ok(Object::String(s.to_owned())),
            LeftParen => {
                Self::parse_expr(tokens)?
            },
            Nil => return Ok(Object::Nil),
            Bool(b) => return Ok(Object::Bool(*b)),
            RightParen => return Err(ParseError::UnexpectedCloseParen),
            // TODO: what should happen on `''a`?
            _ => return Err(ParseError::BadQuote),
        };
        Ok(Object::cons(Object::Symbol("quote".to_string()),
                        Object::cons(quoted, Object::Nil)))
    }

    fn parse_expr<'a>(tokens: &mut Iter<'a, Self>) -> Result<Object, ParseError> {
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
