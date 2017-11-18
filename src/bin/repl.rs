extern crate r7_rs;

use std::io::{stdin, stdout, Write};

fn main() {
    let env = r7_rs::init_env();
    loop {
        // Print prompt
        print!(">> ");
        stdout().flush().unwrap();
        // Read input
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        while input.chars().filter(|&c| c == '(').count()
            > input.chars().filter(|&c| c == ')').count()
        {
            stdin().read_line(&mut input).unwrap();
        }
        let tokens = r7_rs::Parser::parse(&input).unwrap();
        let objects = r7_rs::Token::build_ast(tokens);
        for object in objects {
            if let Some(value) = r7_rs::eval(object, &env) {
                println!("{}", value);
            }
        }
    }
}
