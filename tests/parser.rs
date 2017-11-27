extern crate r7_rs;

use r7_rs::{Parser, Token};

fn run(input: &str) -> Token {
    Parser::parse(input).unwrap().remove(0)
}

#[test]
fn symbol() {
    assert!(run("1").is_integer());
    assert!(run("+1").is_integer());
    assert!(run("-1").is_integer());
    assert!(run("+1+").is_symbol());
    assert!(run("1/2").is_rational());
    assert!(run("+1/2").is_rational());
    assert!(run("-1/2").is_rational());
    assert!(run("1/+2").is_symbol());
    assert!(run("1.0/2").is_symbol());
    assert!(run("1/").is_symbol());
    assert!(run("/2").is_symbol());
    assert!(run("1.0").is_real());
    assert!(run("+1.0").is_real());
    assert!(run("-1.0").is_real());
    assert!(run(".1").is_real());
    assert!(run("+.1").is_real());
    assert!(run("-.1").is_real());
    assert!(run("1e3").is_real());
    assert!(run("+1e3").is_real());
    assert!(run("+1e+3").is_real());
    assert!(run("1.0e+3").is_real());
    assert!(run(".1e+3").is_real());
    assert!(run(".1e3.0").is_symbol());
    assert!(run("1+3i").is_complex_int());
    assert!(run("+1+3i").is_complex_int());
    assert!(run("+1-3i").is_complex_int());
    assert!(run("+1-3").is_symbol());
    assert!(run("1+3/2i").is_complex_rat());
    assert!(run("1/2+3i").is_complex_rat());
    assert!(run("1/2+3/2i").is_complex_rat());
    assert!(run("1/2-3/2i").is_complex_rat());
    assert!(run("+1/2-3/2i").is_complex_rat());
    assert!(run("1.2+3i").is_complex_real());
    assert!(run("1.2+3/2i").is_complex_real());
    assert!(run("1+3.0i").is_complex_real());
    assert!(run("1/2+3.0i").is_complex_real());
    assert!(run("1.0+3.0i").is_complex_real());
    assert!(run("1.0-3.0i").is_complex_real());
    assert!(run("+1.0-3.0i").is_complex_real());
    assert!(run("+1.0-3.0").is_symbol());
    assert!(run("+1.03.0i").is_symbol());
    assert!(run("1030i").is_symbol());
    assert!(run("1+").is_symbol());
    assert!(run("+").is_symbol());
    assert!(run("-").is_symbol());
}
