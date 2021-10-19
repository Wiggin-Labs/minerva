mod ast;
mod error;

pub use self::ast::Ast;
pub use self::error::ParseError;

use Token;
use vm::Value;

use string_interner::get_value;

use std::iter::Peekable;
use std::slice::Iter;

macro_rules! t {
    ($e:expr) => {
        if let Some(e) = $e {
            e
        } else {
            return Err(ParseError::EOF);
        }
    };
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
        match t!(self.tokens.next()) {
            Token::Comment(_) | Token::BlockComment(_) => self._parse(),
            Token::LeftParen => self.parse_expr(),
            Token::Quote => self.parse_quote(false),
            Token::Symbol(s) => Ok(Ast::Ident(*s)),
            t if t.is_primitive() => Ok(Ast::Primitive(t.to_primitive())),
            Token::RightParen => Err(ParseError::UnexpectedCloseParen),
            Token::Pound => self.parse_pound(),
            Token::Dot => Err(ParseError::IllegalUse),
            Token::Quasiquote => unimplemented!(),
            Token::Unquote => unimplemented!(),
            Token::UnquoteSplice => unimplemented!(),
            Token::String(_) | Token::Float(_) | Token::Integer(_) => unreachable!(),
        }
    }

    fn parse_pound(&mut self) -> Result<Ast, ParseError> {
        match t!(self.tokens.next()) {
            Token::Symbol(s) => match get_value(*s).unwrap().as_str() {
                "t" => Ok(Ast::Primitive(Value::Bool(true))),
                "f" => Ok(Ast::Primitive(Value::Bool(false))),
                _ => todo!(),
            }
            //Token::LeftParen => {
            //}
            _ => todo!(),
        }
    }

    fn parse_expr(&mut self) -> Result<Ast, ParseError> {
        match t!(self.tokens.next()) {
            Token::Symbol(s) => match get_value(*s).unwrap().as_str() {
                "define" => self.parse_define(),
                "lambda" => self.parse_lambda(),
                "if" => self.parse_if(),
                "begin" => self.parse_begin(),
                "quote" => self.parse_quote(true),
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
        let mut proc = false;
        let name = match t!(self.tokens.next()) {
            Token::Symbol(s) => *s,
            Token::LeftParen => if let Token::Symbol(s) = t!(self.tokens.next()) {
                proc = true;
                *s
            } else {
                return Err(ParseError::Input);
            },
            _ => return Err(ParseError::Input),
        };

        let value = if proc {
            let mut args = Vec::new();
            loop {
                match t!(self.tokens.next()) {
                    Token::Symbol(s) => args.push(*s),
                    Token::RightParen => break,
                    _ => return Err(ParseError::Input),
                }
            }
            Ast::Lambda{
                args: args,
                body: self.lambda_body()?,
            }
        } else {
            let v = self._parse()?;
            self.read_closer()?;
            v
        };

        Ok(Ast::Define {
            name: name,
            value: Box::new(value)
        })
    }

    fn parse_lambda(&mut self) -> Result<Ast, ParseError> {
        let mut args = vec![];

        if !t!(self.tokens.next()).is_left_paren() {
            return Err(ParseError::Input);
        }

        loop {
            match t!(self.tokens.next()) {
                Token::Symbol(s) => args.push(*s),
                Token::RightParen => break,
                _ => return Err(ParseError::Input),
            }
        }

        let body = self.lambda_body()?;

        Ok(Ast::Lambda { args, body })
    }

    fn lambda_body(&mut self) -> Result<Vec<Ast>, ParseError> {
        match t!(self.tokens.peek()) {
            Token::LeftParen => Ok(self.parse_begin()?.unwrap_begin()),
            Token::RightParen => Err(ParseError::UnexpectedCloseParen),
            _ => {
                let v = vec![self._parse()?];
                self.read_closer()?;
                Ok(v)
            }
        }
    }

    fn parse_if(&mut self) -> Result<Ast, ParseError> {
        let predicate = Box::new(self._parse()?);
        let consequent = Box::new(self._parse()?);
        let alternative = if Token::RightParen == **t!(self.tokens.peek()) {
            Box::new(Ast::Primitive(Value::Void))
        } else {
            Box::new(self._parse()?)
        };

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
            match t!(self.tokens.next()) {
                Token::RightParen => return Ok(Ast::Begin(sequence)),
                Token::LeftParen => sequence.push(self.parse_expr()?),
                Token::Symbol(s) => sequence.push(Ast::Ident(*s)),
                t if t.is_primitive() => sequence.push(Ast::Primitive(t.to_primitive())),
                _ => return Err(ParseError::Input),
            }
        }
    }

    fn parse_application(&mut self, op: Ast) -> Result<Ast, ParseError> {
        let mut args = vec![op];
        loop {
            if t!(self.tokens.peek()).is_right_paren() {
                self.tokens.next();
                return Ok(Ast::Apply(args));
            } else {
                args.push(self._parse()?);
            }
        }
    }

    fn parse_quote(&mut self, read_closer: bool) -> Result<Ast, ParseError> {
        let p = Ast::Primitive(self._parse_quote()?);
        if read_closer {
            self.read_closer()?;
        }
        Ok(p)
    }

    fn _parse_quote(&mut self) -> Result<Value, ParseError> {
        match t!(self.tokens.next()) {
            Token::LeftParen => self.quote_list(),
            Token::Symbol(s) => Ok(Value::Symbol(*s)),
            t if t.is_primitive() => Ok(t.to_primitive()),
            _ => Err(ParseError::Input),
        }
    }

    fn quote_list(&mut self) -> Result<Value, ParseError> {
        let mut parens = 1;
        let mut list_rev = Vec::new();
        while parens != 0 {
            if t!(self.tokens.peek()).is_right_paren() {
                self.tokens.next();
                parens -= 1;
            } else {
                list_rev.push(self._parse_quote()?);
            }
        }

        let mut list = Value::Nil;
        while !list_rev.is_empty() {
            list = Value::Pair(list_rev.pop().unwrap(), list);
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
