extern crate scheme;

use std::io::{stdin, stdout, Write};

fn main() {
    let env = scheme::init_env();
    loop {
        // Print prompt
        print!(">> ");
        stdout().flush().unwrap();
        // Read input
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let tokens = scheme::Parser::parse(&input);
        let objects = scheme::Token::build_ast(tokens);
        for object in objects {
            if let Some(value) = scheme::eval(object, &env) {
                println!("{}", value);
            }
        }
    }
}
