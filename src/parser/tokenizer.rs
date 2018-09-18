use super::ParseError;
use {Ast, CompilePrimitive};

use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, is_enum_variant)]
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
    Integer(String),
    //ComplexExact(Option<String>, Option<String>),
    //ComplexFloating(Option<String>, Option<String>),
    Symbol(String),
}

impl Token {
    /*
    fn is_number(&self) -> bool {
        match self {
            Token::ComplexExact(_, _) | Token::ComplexFloating(_, _) => true,
            _ => false,
        }
    }
    */

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
                Token::Symbol(s) => ast.push(Ast::Ident(s.to_string())),
                Token::RightParen => return Err(ParseError::UnexpectedCloseParen),
                Token::Dot => return Err(ParseError::IllegalUse),
                _ => ast.push(Ast::Primitive(token.to_primitive())),
            }
        }
        Ok(ast)
    }

    fn parse_expr<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        if let Some(token) = tokens.next() {
            match token {
                Token::Symbol(s) => match s.as_str() {
                    "define" => Self::parse_define(tokens),
                    "lambda" => Self::parse_lambda(tokens),
                    "if" => Self::parse_if(tokens),
                    "begin" => Self::parse_begin(tokens),
                    _ => Self::parse_application(Ast::Ident(s.to_string()), tokens),
                }
                Token::LeftParen => {
                    let op = Self::parse_expr(tokens)?;
                    Self::parse_application(op, tokens)
                }
                _ => unimplemented!(),
            }
        } else {
            return Err(ParseError::EOF);
        }
    }

    fn parse_define<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        let name = if let Some(Token::Symbol(s)) = tokens.next() {
            s
        } else {
            return Err(ParseError::Input);
        };

        let value = if let Some(token) = tokens.next() {
            match token {
                Token::LeftParen => Self::parse_expr(tokens)?,
                Token::Symbol(s) => Ast::Ident(s.to_string()),
                Token::Bool(_) => Ast::Primitive(token.to_primitive()),
                Token::Integer(_) => Ast::Primitive(token.to_primitive()),
                Token::String(_) => Ast::Primitive(token.to_primitive()),
                Token::RightParen => return Err(ParseError::UnexpectedCloseParen),
                _ => return Err(ParseError::Input),
            }
        } else {
            return Err(ParseError::EOF);
        };

        if let Some(token) = tokens.next() {
            if token != &Token::RightParen {
                return Err(ParseError::Input);
            }
        } else {
            return Err(ParseError::UnbalancedParen);
        };

        Ok(Ast::Define {
            name: name.to_string(),
            value: Box::new(value)
        })
    }

    fn parse_lambda<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        let mut args = vec![];

        match tokens.next() {
            Some(Token::LeftParen) => (),
            None => return Err(ParseError::EOF),
            _ => return Err(ParseError::Input),
        }

        loop {
            if let Some(token) = tokens.next() {
                match token {
                    Token::Symbol(s) => args.push(s.to_string()),
                    Token::RightParen => break,
                    _ => return Err(ParseError::Input),
                }
            } else {
                return Err(ParseError::EOF);
            }
        }

        let body = match tokens.peek() {
            Some(Token::LeftParen) => Self::parse_begin(tokens)?.unwrap_begin(),
            Some(Token::RightParen) => return Err(ParseError::UnexpectedCloseParen),
            Some(Token::Symbol(s)) => {
                tokens.next();
                match tokens.next() {
                    Some(Token::RightParen) => (),
                    None => return Err(ParseError::EOF),
                    _ => return Err(ParseError::Input),
                }
                vec![Ast::Ident(s.to_string())]
            }
            Some(_) => {
                let token = tokens.next().unwrap();
                let body = match token {
                    Token::Bool(_) => vec![Ast::Primitive(token.to_primitive())],
                    Token::Integer(_) => vec![Ast::Primitive(token.to_primitive())],
                    Token::String(_) => vec![Ast::Primitive(token.to_primitive())],
                    _ => return Err(ParseError::Input),
                };
                match tokens.next() {
                    Some(Token::RightParen) => (),
                    None => return Err(ParseError::EOF),
                    _ => return Err(ParseError::Input),
                }
                body
            }
            None => return Err(ParseError::EOF),
        };

        Ok(Ast::Lambda { args, body })
    }

    fn parse_if<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        macro_rules! if_match {
            ($tokens:ident) => {
                if let Some(token) = $tokens.next() {
                    match token {
                        Token::LeftParen => Self::parse_expr($tokens)?,
                        Token::Bool(_) => Ast::Primitive(token.to_primitive()),
                        Token::Integer(_) => Ast::Primitive(token.to_primitive()),
                        Token::String(_) => Ast::Primitive(token.to_primitive()),
                        Token::Symbol(s) => Ast::Ident(s.to_string()),
                        _ => return Err(ParseError::Input),
                    }
                } else {
                    return Err(ParseError::EOF);
                }
            };
        }

        let predicate = Box::new(if_match!(tokens));
        let consequent = Box::new(if_match!(tokens));
        let alternative = Box::new(if_match!(tokens));

        Ok(Ast::If {
            predicate,
            consequent,
            alternative,
        })
    }

    fn parse_begin<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        let mut sequence = vec![];
        while let Some(token) = tokens.next() {
            match token {
                Token::RightParen => return Ok(Ast::Begin(sequence)),
                Token::LeftParen => sequence.push(Self::parse_expr(tokens)?),
                Token::Symbol(s) => sequence.push(Ast::Ident(s.to_string())),
                Token::Bool(_) => sequence.push(Ast::Primitive(token.to_primitive())),
                Token::Integer(_) => sequence.push(Ast::Primitive(token.to_primitive())),
                Token::String(_) => sequence.push(Ast::Primitive(token.to_primitive())),
                _ => return Err(ParseError::Input),
            }
        }

        Err(ParseError::EOF)
    }

    fn parse_application<'a>(op: Ast, tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Ast, ParseError> {
        let mut args = vec![op];
        while let Some(token) = tokens.next() {
            match token {
                Token::RightParen => return Ok(Ast::Apply(args)),
                Token::LeftParen => args.push(Self::parse_expr(tokens)?),
                Token::Symbol(s) => args.push(Ast::Ident(s.to_string())),
                Token::Bool(_) => args.push(Ast::Primitive(token.to_primitive())),
                Token::Integer(_) => args.push(Ast::Primitive(token.to_primitive())),
                Token::String(_) => args.push(Ast::Primitive(token.to_primitive())),
                _ => return Err(ParseError::Input),
            }
        }

        Err(ParseError::EOF)
    }

    fn to_primitive(&self) -> CompilePrimitive {
        match self {
            Token::Nil => CompilePrimitive::Nil,
            Token::Bool(b) => CompilePrimitive::Bool(*b),
            Token::String(s) => CompilePrimitive::String(s.to_string()),
            Token::Integer(i) => CompilePrimitive::Integer(i.parse().unwrap()),
            _ => unreachable!(),
        }
    }

/*
    fn to_object(&self) -> Sexp {
        match self {
            Token::Nil => Sexp::Nil,
            Token::Bool(b) => Sexp::Bool(*b),
            Token::String(s) => Sexp::String(s.to_owned()),
            num if self.is_number() => Sexp::Number(::Number::from_token(num)),
            Token::Symbol(s) => Sexp::Symbol(s.to_owned()),
            _ => unreachable!(),
        }
    }

    pub fn build_ast(tokens: Vec<Self>) -> Result<Vec<Sexp>, ParseError> {
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
                Quasiquote => {
                    let list = Self::parse_quasiquote(&mut tokens)?;
                    exprs.push(list);
                }
                Unquote => return Err(ParseError::IllegalUse),
                UnquoteSplice => return Err(ParseError::IllegalUse),
                _ => exprs.push(token.to_object()),
            }
        }

        Ok(exprs)
    }

    fn parse_quote<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Sexp, ParseError> {
        use self::Token::*;
        let next = if let Some(t) = tokens.next() {
            t
        } else {
            return Err(ParseError::BadQuote);
        };

        let quoted = match next {
            Symbol(s) => Sexp::Symbol(s.to_owned()),
            LeftParen => {
                Self::parse_expr(tokens)?
            },
            Dot => return Err(ParseError::IllegalUse),
            Unquote => return Err(ParseError::IllegalUse),
            UnquoteSplice => return Err(ParseError::IllegalUse),
            RightParen => return Err(ParseError::UnexpectedCloseParen),
            Quote => Self::parse_quote(tokens)?,
            _ => return Ok(next.to_object()),
        };
        Ok(Sexp::cons(Sexp::Symbol("quote".to_string()),
                        Sexp::cons(quoted, Sexp::Nil)))
    }

    fn parse_quasiquote<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Sexp, ParseError> {
        use self::Token::*;
        let next = if let Some(t) = tokens.next() {
            t
        } else {
            return Err(ParseError::BadQuote);
        };

        let quoted = match next {
            Symbol(s) => Sexp::Symbol(s.to_owned()),
            LeftParen => {
                Self::parse_expr(tokens)?
            },
            Dot => return Err(ParseError::IllegalUse),
            RightParen => return Err(ParseError::UnexpectedCloseParen),
            Unquote => {
                // TODO parse next expr
                Sexp::cons(Sexp::Symbol("unquote".to_string()),
                             Sexp::cons(Sexp::Nil, Sexp::Nil))
            }
            UnquoteSplice => return Err(ParseError::IllegalUse),
            Quote => Self::parse_quote(tokens)?,
            _ => return Ok(next.to_object()),
        };

        Ok(Sexp::cons(Sexp::Symbol("quasiquote".to_string()),
                        Sexp::cons(quoted, Sexp::Nil)))
    }

    fn parse_expr<'a>(tokens: &mut Peekable<Iter<'a, Self>>) -> Result<Sexp, ParseError> {
        use self::Token::*;
        let mut parens = 1;
        let mut stack = Vec::new();
        let mut list = Sexp::Nil;

        while let Some(token) = tokens.next() {
            match token {
                LeftParen => {
                    parens += 1;
                    let mut new = Sexp::Nil;
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
                }
                Quasiquote => {
                    let l = Self::parse_quasiquote(tokens)?;
                    list = list.push(l);
                }
                Nil => list = list.push(Sexp::Nil),
                Bool(b) => list = list.push(Sexp::Bool(*b)),
                String(s) => list = list.push(Sexp::String(s.to_owned())),
                Symbol(s) => list = list.push(Sexp::Symbol(s.to_owned())),
                num if token.is_number() =>
                    list = list.push(Sexp::Number(::Number::from_token(num))),
                _ => unreachable!(),
            }
        }

        Err(ParseError::UnbalancedParen)
    }
    */
}
