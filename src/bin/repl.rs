extern crate akuma;
#[cfg(feature="profile")]
extern crate flame;
extern crate vm;

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
            println!("{:?}", result);
        }
    }

    #[cfg(feature="profile")]
    flame::dump_html(&mut std::fs::File::create("profile2.html").unwrap()).unwrap();
}
