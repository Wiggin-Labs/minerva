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
        let tokens = match r7_rs::Parser::parse(&input) {
            Ok(t) => t,
            Err(e) => {
                println!("ERROR: {}", e);
                continue;
            }
        };
        let objects = match r7_rs::Token::build_ast(tokens) {
            Ok(o) => o,
            Err(e) => {
                println!("ERROR: {}", e);
                continue;
            }
        };

        for object in objects {
            if let Some(value) = r7_rs::eval(object, &env) {
                println!("{}", value);
            }
        }
    }
}
