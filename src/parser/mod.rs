mod error;

pub use self::error::ParseError;

use {Ast, CompilePrimitive};

use string_interner::{INTERNER, Symbol};

use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, Clone, PartialEq, PartialOrd, is_enum_variant)]
pub enum Token {
    Comment(String),
    BlockComment(String),
    LeftParen,
    RightParen,
    Dot,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplice,
    Nil,
    Bool(bool),
    String(String),
    Integer(i32),
    Float(f64),
    //ComplexExact(Option<String>, Option<String>),
    //ComplexFloating(Option<String>, Option<String>),
    Symbol(Symbol),
}

impl Token {
    pub fn build_ast(tokens: Vec<Self>) -> Result<Vec<Ast>, ParseError> {
        let mut ast = vec![];
        let mut tokens = tokens.iter().peekable();
        while let Some(token) = tokens.next() {
            match token {
                Token::Comment(_) | Token::BlockComment(_) => {},
                Token::LeftParen => {
                    let list = Self::parse_expr(&mut tokens)?;
                    ast.push(list);
                }
                Token::Symbol(s) => ast.push(Ast::Ident(*s)),
                Token::RightParen => return Err(ParseError::UnexpectedCloseParen),
                Token::Dot => return Err(ParseError::IllegalUse),
                _ => ast.push(Ast::Primitive(token.to_primitive())),
            }
        }
        Ok(ast)
    }

    fn parse_expr<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        match tokens.next()? {
            Token::Symbol(s) => match INTERNER.lock().unwrap().get_value(*s).unwrap().as_str() {
                "define" => Self::parse_define(tokens),
                "lambda" => Self::parse_lambda(tokens),
                "if" => Self::parse_if(tokens),
                "begin" => Self::parse_begin(tokens),
                _ => Self::parse_application(Ast::Ident(*s), tokens),
            }
            Token::LeftParen => {
                let op = Self::parse_expr(tokens)?;
                Self::parse_application(op, tokens)
            }
            _ => unimplemented!(),
        }
    }

    fn parse_define<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        let name = if let Token::Symbol(s) = tokens.next()? {
            s
        } else {
            return Err(ParseError::Input);
        };

        let value = match tokens.next()? {
            Token::LeftParen => Self::parse_expr(tokens)?,
            Token::Symbol(s) => Ast::Ident(*s),
            token @ Token::Bool(_) => Ast::Primitive(token.to_primitive()),
            token @ Token::Integer(_) => Ast::Primitive(token.to_primitive()),
            token @ Token::String(_) => Ast::Primitive(token.to_primitive()),
            Token::RightParen => return Err(ParseError::UnexpectedCloseParen),
            _ => return Err(ParseError::Input),
        };

        if let Some(token) = tokens.next() {
            if token != &Token::RightParen {
                return Err(ParseError::Input);
            }
        } else {
            return Err(ParseError::UnbalancedParen);
        };

        Ok(Ast::Define {
            name: *name,
            value: Box::new(value)
        })
    }

    fn parse_lambda<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        let mut args = vec![];

        if !tokens.next()?.is_left_paren() {
            return Err(ParseError::Input);
        }

        loop {
            match tokens.next()? {
                Token::Symbol(s) => args.push(*s),
                Token::RightParen => break,
                _ => return Err(ParseError::Input),
            }
        }

        let body = match tokens.peek()? {
            Token::LeftParen => Self::parse_begin(tokens)?.unwrap_begin(),
            Token::RightParen => return Err(ParseError::UnexpectedCloseParen),
            Token::Symbol(s) => {
                tokens.next();
                if !tokens.next()?.is_right_paren() {
                    return Err(ParseError::Input);
                }
                vec![Ast::Ident(*s)]
            }
            _ => {
                let token = tokens.next().unwrap();
                let body = match token {
                    Token::Bool(_) => vec![Ast::Primitive(token.to_primitive())],
                    Token::Integer(_) => vec![Ast::Primitive(token.to_primitive())],
                    Token::String(_) => vec![Ast::Primitive(token.to_primitive())],
                    _ => return Err(ParseError::Input),
                };
                if !tokens.next()?.is_right_paren() {
                    return Err(ParseError::Input);
                }
                body
            }
        };

        Ok(Ast::Lambda { args, body })
    }

    fn parse_if<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        macro_rules! if_match {
            ($tokens:ident) => {
                match $tokens.next()? {
                    Token::LeftParen => Self::parse_expr($tokens)?,
                    token @ Token::Bool(_) => Ast::Primitive(token.to_primitive()),
                    token @ Token::Integer(_) => Ast::Primitive(token.to_primitive()),
                    token @ Token::String(_) => Ast::Primitive(token.to_primitive()),
                    Token::Symbol(s) => Ast::Ident(*s),
                    _ => return Err(ParseError::Input),
                }
            };
        }

        let predicate = Box::new(if_match!(tokens));
        let consequent = Box::new(if_match!(tokens));
        let alternative = Box::new(if_match!(tokens));

        if let Some(token) = tokens.next() {
            if token != &Token::RightParen {
                return Err(ParseError::Input);
            }
        } else {
            return Err(ParseError::UnbalancedParen);
        };

        Ok(Ast::If {
            predicate,
            consequent,
            alternative,
        })
    }

    fn parse_begin<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        let mut sequence = vec![];
        loop {
            match tokens.next()? {
                Token::RightParen => return Ok(Ast::Begin(sequence)),
                Token::LeftParen => sequence.push(Self::parse_expr(tokens)?),
                Token::Symbol(s) => sequence.push(Ast::Ident(*s)),
                token @ Token::Bool(_) => sequence.push(Ast::Primitive(token.to_primitive())),
                token @ Token::Integer(_) => sequence.push(Ast::Primitive(token.to_primitive())),
                token @ Token::String(_) => sequence.push(Ast::Primitive(token.to_primitive())),
                _ => return Err(ParseError::Input),
            }
        }
    }

    fn parse_application<'a>(op: Ast, tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        let mut args = vec![op];
        loop {
            match tokens.next()? {
                Token::RightParen => return Ok(Ast::Apply(args)),
                Token::LeftParen => args.push(Self::parse_expr(tokens)?),
                Token::Symbol(s) => args.push(Ast::Ident(*s)),
                token @ Token::Bool(_) => args.push(Ast::Primitive(token.to_primitive())),
                token @ Token::Integer(_) => args.push(Ast::Primitive(token.to_primitive())),
                token @ Token::String(_) => args.push(Ast::Primitive(token.to_primitive())),
                _ => return Err(ParseError::Input),
            }
        }
    }

    fn to_primitive(&self) -> CompilePrimitive {
        match self {
            Token::Nil => CompilePrimitive::Nil,
            Token::Bool(b) => CompilePrimitive::Bool(*b),
            Token::String(s) => CompilePrimitive::String(s.to_string()),
            Token::Integer(i) => CompilePrimitive::Integer(*i),
            Token::Float(i) => CompilePrimitive::Float(*i),
            _ => unreachable!(),
        }
    }
}
