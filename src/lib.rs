#[macro_use]
extern crate derive_is_enum_variant;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate string_interner;
extern crate vm;

mod compiler;
mod error;
mod optimize;
mod parser;
mod tokenizer;

pub use compiler::{Ast, compile};
pub use error::Error;
pub use optimize::{IR, optimize, output_asm};
pub use parser::{Parser, ParseError, Token};
pub use tokenizer::Tokenizer;
