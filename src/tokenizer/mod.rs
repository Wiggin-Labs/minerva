mod token;

pub use self::token::Token;

use ParseError;

use regex::Regex;
use string_interner::get_symbol;

use std::iter::Peekable;
use std::str::Chars;

type ParseResult = Result<(), ParseError>;

pub struct Tokenizer<'a> {
    position: usize,
    input: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn tokenize(input: &'a str) -> Result<Vec<Token>, ParseError> {
        let input = input.chars().peekable();
        let mut tokenizer = Tokenizer {
            position: 0,
            input: input,
            tokens: Vec::new(),
        };
        tokenizer._tokenize()?;

        Ok(tokenizer.tokens)
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
        self.input.peek().copied()
    }

    fn _tokenize(&mut self) -> ParseResult {
        while let Some(c) = self.next() {
            match c {
                c if is_pair_start(c) => self.tokens.push(Token::LeftParen),
                c if is_pair_end(c) => self.tokens.push(Token::RightParen),
                '\'' => self.tokens.push(Token::Quote),
                '`' => self.tokens.push(Token::Quasiquote),
                ',' => match self.peek() {
                    Some('@') => {
                        self.next();
                        self.tokens.push(Token::UnquoteSplice);
                    }
                    _ => self.tokens.push(Token::Unquote),
                },
                '"' => self.tokenize_string()?,
                '|' => self.tokenize_identifier(String::new(), true)?,
                ';' => self.tokenize_comment(c)?,
                '#' => {
                    match self.peek() {
                        Some('|') => {
                            self.next();
                            self.tokenize_block_comment()?;
                        }
                        _ => self.tokens.push(Token::Pound),
                    }
                }
                c if c.is_whitespace() => {}
                '.' => match self.peek() {
                    Some(c) => match c {
                        c if is_delimiter(c) => self.tokens.push(Token::Dot),
                        _ => self.tokenize_ambiguous('.')?,
                    },
                    None => self.tokens.push(Token::Dot),
                },
                '0' ..= '9' | '+' | '-' => self.tokenize_ambiguous(c)?,
                _ => {
                    let mut buf = String::new();
                    match c {
                        '\\' => match self.next() {
                            Some(c) => buf.push(c),
                            None => return Err(ParseError::EOF),
                        },
                        _ => buf.push(c),
                    }
                    self.tokenize_identifier(buf, false)?;
                }
            }
        }
        Ok(())
    }

    fn tokenize_ambiguous(&mut self, c: char) -> ParseResult {
        let mut buf = String::new();
        buf.push(c);

        while let Some(c) = self.next() {
            match c {
                '0' ..= '9' | '+' | '-' | '/' | '.' | 'e' | 'i' => buf.push(c),
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
                        return self.tokenize_identifier(buf, false);
                    }
                    None => return Err(ParseError::EOF),
                },
                _ => {
                    buf.push(c);
                    return self.tokenize_identifier(buf, c == '|');
                }
            }
        }
        self.distinguish_ambiguous(buf)
    }

    fn distinguish_ambiguous(&mut self, buf: String) -> ParseResult {
        use std::sync::LazyLock;

        //const _RAT: &str = r"\d+(?:/\d+)?";
        const _REAL: &str = r"\d*\.?\d+(?:[eE][-+]?\d+)?";
        static INTEGER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([+-]?\d+)$").unwrap());
        static FLOAT: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r"^([+-]?{})$", _REAL)).unwrap());
        //static COMPLEX_RAT: SyncLazy<Regex> = SyncLazy::new(|| Regex::new(&format!("^([+-]?{})?(?:([+-](?:{0})?)i)?$", _RAT)).unwrap());
        //static COMPLEX_REAL: SyncLazy<Regex> = SyncLazy::new(|| Regex::new(&format!("^([+-]?(?:{}|{}))?(?:([+-](?:{0}|{1})?)i)?$", _REAL, _RAT)).unwrap());


        /*
        if COMPLEX_RAT.is_match(&buf) {
            let captures = COMPLEX_RAT.captures(&buf).unwrap();
            let real = captures.get(1).map(|s| s.as_str().to_owned());
            let imaginary = captures.get(2).map(|s| s.as_str().to_owned());
            self.tokens.push(Token::ComplexExact(real, imaginary));
        } else if COMPLEX_REAL.is_match(&buf) {
            let captures = COMPLEX_REAL.captures(&buf).unwrap();
            let real = captures.get(1).map(|s| s.as_str().to_owned());
            let imaginary = captures.get(2).map(|s| s.as_str().to_owned());
            self.tokens.push(Token::ComplexFloating(real, imaginary));
            */
        if INTEGER.is_match(&buf) {
            let captures = INTEGER.captures(&buf).unwrap();
            let n = captures.get(1).map(|s| s.as_str().to_owned()).unwrap();
            self.tokens.push(Token::Integer(n.parse().unwrap()));
        } else if FLOAT.is_match(&buf) {
            let captures = FLOAT.captures(&buf).unwrap();
            let n = captures.get(1).map(|s| s.as_str().to_owned()).unwrap();
            self.tokens.push(Token::Float(n.parse().unwrap()));
        } else {
            self.tokens.push(Token::Symbol(get_symbol(buf)));
        }
        Ok(())
    }

    fn tokenize_identifier(&mut self, mut buf: String, mut in_bar: bool) -> ParseResult {
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
                    self.tokens.push(Token::Symbol(get_symbol(buf)));
                    return match c {
                        c if c.is_whitespace() => Ok(()),
                        c if is_pair_start(c) => Ok(self.tokens.push(Token::LeftParen)),
                        c if is_pair_end(c) => Ok(self.tokens.push(Token::RightParen)),
                        '"' => self.tokenize_string(),
                        ';' => self.tokenize_comment(c),
                        _ => panic!("Parser error"),
                    };
                },
                _ => buf.push(c),
            }
        }
        self.tokens.push(Token::Symbol(get_symbol(buf)));
        Ok(())
    }

    pub fn tokenize_string(&mut self) -> ParseResult {
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
                    return Err(ParseError::InString);
                },
                '"' => {
                    self.tokens.push(Token::String(buf));
                    return Ok(());
                }
                _ => buf.push(c),
            }
        }
        Err(ParseError::InString)
    }

    fn tokenize_block_comment(&mut self) -> ParseResult {
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
    fn tokenize_comment(&mut self, _c: char) -> ParseResult {
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
    matches!(c, '(' | '[' | '{')
}

fn is_pair_end(c: char) -> bool {
    matches!(c, ')' | ']' | '}')
}
