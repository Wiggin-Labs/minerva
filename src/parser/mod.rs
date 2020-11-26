mod error;

pub use self::error::ParseError;

use {Ast};
use vm::Value;

use string_interner::{get_value, Symbol};

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
    fn to_primitive(&self) -> Value {
        match self {
            Token::Nil => Value::Nil,
            Token::Bool(b) => Value::Bool(*b),
            Token::String(s) => Value::String(s.to_string()),
            Token::Integer(i) => Value::Integer(*i),
            Token::Float(i) => Value::Float(*i),
            _ => unreachable!(),
        }
    }
}

pub struct Parser<'a> {
    ast: Vec<Ast>,
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn parse(tokens: Vec<Token>) -> Result<Vec<Ast>, ParseError> {
        let ast = vec![];
        let tokens = tokens.iter().peekable();
        let mut parser = Parser {
            ast: ast,
            tokens: tokens,
        };
        while parser.tokens.peek().is_some() {
            let p = parser._parse()?;
            parser.ast.push(p);
        }

        Ok(parser.ast)
    }

    fn _parse(&mut self) -> Result<Ast, ParseError> {
        match self.tokens.next()? {
            Token::Comment(_) | Token::BlockComment(_) => self._parse(),
            Token::LeftParen => self.parse_expr(),
            Token::Quote => self.parse_quote(),
            Token::Symbol(s) => Ok(Ast::Ident(*s)),
            t @ Token::Nil | t @ Token::Bool(_) | t @ Token::String(_) |
                t @ Token::Integer(_) | t @ Token::Float(_) => Ok(Ast::Primitive(t.to_primitive())),
            Token::RightParen => Err(ParseError::UnexpectedCloseParen),
            Token::Dot => Err(ParseError::IllegalUse),
            Token::Quasiquote => unimplemented!(),
            Token::Unquote => unimplemented!(),
            Token::UnquoteSplice => unimplemented!(),
        }
    }

    fn parse_expr(&mut self) -> Result<Ast, ParseError> {
        match self.tokens.next()? {
            Token::Symbol(s) => match get_value(*s).unwrap().as_str() {
                "define" => self.parse_define(),
                "lambda" => self.parse_lambda(),
                "if" => self.parse_if(),
                "begin" => self.parse_begin(),
                _ => self.parse_application(Ast::Ident(*s)),
            }
            Token::LeftParen => {
                let op = self.parse_expr()?;
                self.parse_application(op)
            }
            _ => unimplemented!(),
        }
    }

    fn parse_define(&mut self) -> Result<Ast, ParseError> {
        let name = if let Token::Symbol(s) = self.tokens.next()? {
            s
        } else {
            return Err(ParseError::Input);
        };

        let value = self._parse()?;
        self.read_closer()?;

        Ok(Ast::Define {
            name: *name,
            value: Box::new(value)
        })
    }

    fn parse_lambda(&mut self) -> Result<Ast, ParseError> {
        let mut args = vec![];

        if !self.tokens.next()?.is_left_paren() {
            return Err(ParseError::Input);
        }

        loop {
            match self.tokens.next()? {
                Token::Symbol(s) => args.push(*s),
                Token::RightParen => break,
                _ => return Err(ParseError::Input),
            }
        }

        let body = match self.tokens.peek()? {
            Token::LeftParen => self.parse_begin()?.unwrap_begin(),
            Token::RightParen => return Err(ParseError::UnexpectedCloseParen),
            _ => {
                let v = vec![self._parse()?];
                self.read_closer()?;
                v
            }
        };

        Ok(Ast::Lambda { args, body })
    }

    fn parse_if(&mut self) -> Result<Ast, ParseError> {
        let predicate = Box::new(self._parse()?);
        let consequent = Box::new(self._parse()?);
        let alternative = Box::new(self._parse()?);

        self.read_closer()?;

        Ok(Ast::If {
            predicate,
            consequent,
            alternative,
        })
    }

    fn parse_begin(&mut self) -> Result<Ast, ParseError> {
        let mut sequence = vec![];
        loop {
            match self.tokens.next()? {
                Token::RightParen => return Ok(Ast::Begin(sequence)),
                Token::LeftParen => sequence.push(self.parse_expr()?),
                Token::Symbol(s) => sequence.push(Ast::Ident(*s)),
                token @ Token::Bool(_) => sequence.push(Ast::Primitive(token.to_primitive())),
                token @ Token::Integer(_) => sequence.push(Ast::Primitive(token.to_primitive())),
                token @ Token::String(_) => sequence.push(Ast::Primitive(token.to_primitive())),
                _ => return Err(ParseError::Input),
            }
        }
    }

    fn parse_application(&mut self, op: Ast) -> Result<Ast, ParseError> {
        let mut args = vec![op];
        loop {
            if self.tokens.peek()?.is_right_paren() {
                self.tokens.next();
                return Ok(Ast::Apply(args));
            } else {
                args.push(self._parse()?);
            }
        }
    }

    fn parse_quote(&mut self) -> Result<Ast, ParseError> {
        Ok(Ast::Primitive(self._parse_quote()?))
    }
    fn _parse_quote(&mut self) -> Result<Value, ParseError> {
        match self.tokens.next()? {
            Token::LeftParen => self.quote_list(),
            Token::Symbol(s) => Ok(Value::Symbol(*s)),
            t @ Token::Nil | t @ Token::Bool(_) | t @ Token::String(_) |
                t @ Token::Integer(_) | t @ Token::Float(_) => Ok(t.to_primitive()),
            _ => Err(ParseError::Input),
        }
    }

    fn quote_list(&mut self) -> Result<Value, ParseError> {
        let mut parens = 1;
        let mut list_rev = Vec::new();
        while parens != 0 {
            if self.tokens.peek()?.is_right_paren() {
                self.tokens.next();
                parens -= 1;
            } else if self.tokens.peek()?.is_left_paren() {
                parens += 1;
                list_rev.push(self._parse_quote()?);
            } else {
                list_rev.push(self._parse_quote()?);
            }
        }

        let mut list = Value::Nil;
        for i in 0..list_rev.len() {
            list = Value::Pair(list_rev[list_rev.len()-1-i], list);
        }

        Ok(list)
    }

    fn read_closer(&mut self) -> Result<(), ParseError> {
        if let Some(token) = self.tokens.next() {
            if token != &Token::RightParen {
                return Err(ParseError::Input);
            }
        } else {
            return Err(ParseError::UnbalancedParen);
        }
        Ok(())
    }
}
