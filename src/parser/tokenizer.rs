use super::ParseError;
use Object;

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

    pub fn build_list(tokens: Vec<Self>) -> Result<Vec<Object>, ParseError> {
        use self::Token::*;
        let mut exprs = Vec::new();
        let mut tokens = tokens.iter();
        let mut stack = Vec::new();

        let mut parens = 0;
        while let Some(token) = tokens.next() {
            match token {
                LeftParen => {
                }
            }
        }
        Ok(exprs)
    }

    pub fn build_ast(tokens: Vec<Self>) -> Result<Vec<Object>, ParseError> {
        use self::Token::*;
        let mut exprs = Vec::new();
        let mut tokens = tokens.iter();
        while let Some(token) = tokens.next() {
            match token {
                LeftParen => {
                    let mut list = Object::Nil;
                    Self::parse_expr(&mut tokens, &mut list)?;
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
                let mut list = Object::Nil;
                Self::parse_expr(tokens, &mut list)?;
                list
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

    fn parse_expr<'a>(tokens: &mut Iter<'a, Self>, list: &mut Object) -> Result<(), ParseError> {
        use self::Token::*;
        let mut parens = 1;
        let mut stack = Vec::new();
        let mut l = Object::Nil;

        while parens > 0 {
            let token = if let Some(t) = tokens.next() {
                t
            } else {
                return Err(ParseError::UnbalancedParen);
            };

            match token {
                LeftParen => {
                    parens += 1;
                    let mut new = Object::Nil;
                    mem::swap(new, l);
                    stack.push(new);
                }
                RightParen => {
                    parens -= 1;
                }
            }
        }
        while let Some(token) = tokens.next() {
            match token {
                LeftParen => {
                    let mut l = Object::Nil;
                    Self::parse_expr(tokens, &mut l)?;
                    *list = list.push(l);
                },
                RightParen => {
                    parens -= 1;
                    break;
                }
                Quote => {
                    let l = Self::parse_quote(tokens)?;
                    *list = list.push(l);
                },
                Nil => *list = list.push(Object::Nil),
                Bool(b) => *list = list.push(Object::Bool(*b)),
                String(s) => *list = list.push(Object::String(s.to_owned())),
                Symbol(s) => *list = list.push(Object::Symbol(s.to_owned())),
                num if token.is_number() =>
                    *list = list.push(Object::Number(::Number::from_token(num))),
                _ => unreachable!(),
            }
        }

        if parens != 0 {
            Err(ParseError::UnbalancedParen)
        } else {
            Ok(())
        }
    }
}
