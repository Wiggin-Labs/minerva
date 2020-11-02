extern crate akuma;
#[cfg(feature="profile")]
extern crate flame;
extern crate vm;

use akuma::ParseError;
use vm::{assemble, init_env, Register, VM};

use std::io::{stdin, stdout, Write};

fn main() {
    let mut vm = VM::new();
    let env = init_env(&mut vm);
    vm.assign_environment(env);

    loop {
        // Print prompt
        print!(">> ");
        stdout().flush().unwrap();
        // Read input
        let mut input = String::new();
        if 0 == stdin().read_line(&mut input).unwrap() {
            // EOF
            return;
        }
        let mut tokens = match akuma::Parser::parse(&input) {
            Ok(t) => t,
            Err(e) => match e {
                ParseError::InString => Vec::new(),
                _ => {
                    println!("ERROR: {}", e);
                    continue;
                }
            },
        };

        loop {
            if tokens.is_empty() ||
                tokens.iter().filter(|&t| t.is_left_paren()).count()
                > tokens.iter().filter(|&t| t.is_right_paren()).count()
            {
                if 0 == stdin().read_line(&mut input).unwrap() {
                    // EOF
                    return;
                }
                tokens = match akuma::Parser::parse(&input) {
                    Ok(t) => t,
                    Err(e) => match e {
                        ParseError::InString => Vec::new(),
                        _ => {
                            println!("ERROR: {}", e);
                            continue;
                        }
                    },
                };
            } else {
                break;
            }
        }

        if "exit\n" == input {
            break;
        }

        let ast = match akuma::Token::build_ast(tokens) {
            Ok(o) => o,
            Err(e) => {
                println!("ERROR: {}", e);
                continue;
            }
        };

        for ast in ast {
            let asm = akuma::compile(ast);
            vm.load_code(assemble(asm));
            vm.run();
            let result = vm.load_register(Register::A);
            println!("{}", result);
        }
    }

    #[cfg(feature="profile")]
    flame::dump_html(&mut std::fs::File::create("profile2.html").unwrap()).unwrap();
}
