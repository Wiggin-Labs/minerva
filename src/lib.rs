#![feature(try_trait)]
#![cfg_attr(feature="profile", feature(plugin, custom_attribute))]
#![cfg_attr(feature="profile", plugin(flamer))]
#[cfg(feature="profile")]
extern crate flame;

#[macro_use]
extern crate derive_is_enum_variant;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate string_interner;
extern crate vm;

mod compiler;
mod error;
mod parser;
mod tokenizer;

pub use compiler::{Ast, compile, CompilePrimitive};
pub use error::Error;
pub use parser::{Parser, ParseError, Token};
pub use tokenizer::Tokenizer;
