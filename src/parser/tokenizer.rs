use Object;

use num::BigInt;

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
                Number(i) => exprs.push(Object::Number(i.parse::<BigInt>().unwrap())),
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
                return Object::Number(i.parse::<BigInt>().unwrap());
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
                Number(i) => *list = list.push(Object::Number(i.parse::<BigInt>().unwrap())),
            }
        }
        assert!(parens == 0);
    }
}
