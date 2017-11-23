extern crate afl;
extern crate r7_rs;

fn main() {
    let env = r7_rs::init_env();
    afl::read_stdio_string(|s| {
        if let Ok(tokens) = r7_rs::Parser::parse(&s) {
            if let Ok(objects) = r7_rs::Token::build_ast(tokens) {
                for object in objects {
                    r7_rs::eval(object, &env);
                }
            }
        }
    });
}
