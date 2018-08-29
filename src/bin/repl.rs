extern crate akuma;
#[cfg(feature="profile")]
extern crate flame;

use std::io::{stdin, stdout, Write};

fn main() {
    let env = akuma::init_env();
    /*
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

        if "exit\n" == input {
            break;
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
    */

    let input = "(define (fib n) (cond ((= n 0) 1) ((= n 1) 1) (else (+ (fib (- n 1)) (fib (- n 2))))))";
    let tokens = akuma::Parser::parse(&input).unwrap();
    let sexps = akuma::Token::build_ast(tokens).unwrap();
    for sexp in sexps {
        println!("{}", akuma::eval(sexp, &env));
    }
    let input = "(fib 5)";
    let tokens = akuma::Parser::parse(&input).unwrap();
    let sexps = akuma::Token::build_ast(tokens).unwrap();
    for sexp in sexps {
        println!("{}", akuma::eval(sexp, &env));
    }

    #[cfg(feature="profile")]
    flame::dump_html(&mut std::fs::File::create("profile2.html").unwrap()).unwrap();
}
