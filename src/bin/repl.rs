extern crate akuma;

use std::io::{stdin, stdout, Write};

fn main() {
    let env = akuma::init_env();
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
        let tokens = match akuma::Parser::parse(&input) {
            Ok(t) => t,
            Err(e) => {
                println!("ERROR: {}", e);
                continue;
            }
        };
        let objects = match akuma::Token::build_ast(tokens) {
            Ok(o) => o,
            Err(e) => {
                println!("ERROR: {}", e);
                continue;
            }
        };

        for object in objects {
            println!("{}", akuma::eval(object, &env));
        }
    }
}
