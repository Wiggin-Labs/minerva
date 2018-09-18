extern crate akuma;
extern crate ramp;

/*
use akuma::{Parser, Token, Sexp, ComplexExact, ComplexFloating, Number};
use ramp::Int;
use ramp::rational::Rational;

fn new_rat(numerator: i64, denominator: i64) -> Rational {
    Rational::new(Int::from(numerator), Int::from(denominator))
}

#[inline]
fn rat_zero() -> Rational {
    Rational::new(Int::zero(), Int::one())
}

#[inline]
fn rat_one() -> Rational {
    Rational::new(Int::one(), Int::one())
}

fn new_exact(real: Rational, imaginary: Rational) -> Number {
    Number::Exact(ComplexExact::new(real, imaginary))
}

fn new_float(real: f64, imaginary: f64) -> Number {
    Number::Floating(ComplexFloating::new(real, imaginary))
}

fn run(input: &str) -> Sexp {
    let tokens = Parser::parse(input).unwrap();
    Token::build_ast(tokens).unwrap().remove(0)
}

macro_rules! assert_num {
    ($l:expr, $r:expr) => {
        assert_eq!($l, Sexp::Number($r))
    };
}

#[test]
fn numbers() {
    assert_num!(run("1"), Number::from(1));
    assert_num!(run("+1"), Number::from(1));
    assert_num!(run("-1"), Number::from(-1));
    assert_num!(run("1/2"), Number::from(new_rat(1, 2)));
    assert_num!(run("+1/2"), Number::from(new_rat(1, 2)));
    assert_num!(run("-1/2"), Number::from(new_rat(-1, 2)));
    assert_num!(run("1.0"), Number::from(1.0));
    assert_num!(run("+1.0"), Number::from(1.0));
    assert_num!(run("-1.0"), Number::from(-1.0));
    assert_num!(run(".1"), Number::from(0.1));
    assert_num!(run("+.1"), Number::from(0.1));
    assert_num!(run("-.1"), Number::from(-0.1));
    assert_num!(run("+1e3"), Number::from(1e3));
    assert_num!(run("+1e+3"), Number::from(1e3));
    assert_num!(run("1.0e+3"), Number::from(1e3));
    assert_num!(run(".1e+3"), Number::from(0.1e3));
    assert_num!(run("1+3i"), new_exact(rat_one(), new_rat(3, 1)));
    assert_num!(run("1+i"), new_exact(rat_one(), rat_one()));
    assert_num!(run("+i"), new_exact(rat_zero(), rat_one()));
    assert_num!(run("+3i"), new_exact(rat_zero(), new_rat(3, 1)));
    assert_num!(run("+1+3i"), new_exact(rat_one(), new_rat(3, 1)));
    assert_num!(run("+1-3i"), new_exact(rat_one(), new_rat(-3, 1)));
    assert_num!(run("1+3/2i"), new_exact(rat_one(), new_rat(3, 2)));
    assert_num!(run("1/2+3i"), new_exact(new_rat(1, 2), new_rat(3, 1)));
    assert_num!(run("1/2+i"), new_exact(new_rat(1, 2), new_rat(1, 1)));
    assert_num!(run("1/2-i"), new_exact(new_rat(1, 2), new_rat(-1, 1)));
    assert_num!(run("1/2+3/2i"), new_exact(new_rat(1, 2), new_rat(3, 2)));
    assert_num!(run("1/2-3/2i"), new_exact(new_rat(1, 2), new_rat(-3, 2)));
    assert_num!(run("+1/2-3/2i"), new_exact(new_rat(1, 2), new_rat(-3, 2)));
    assert_num!(run("1.2+3i"), new_float(1.2, 3.0));
    assert_num!(run("1.2+3/2i"), new_float(1.2, 3.0 / 2.0));
    assert_num!(run("1+3.0i"), new_float(1.0, 3.0));
    assert_num!(run("1/2+3.0i"), new_float(1.0 / 2.0, 3.0));
    assert_num!(run("1.0+3.0i"), new_float(1.0, 3.0));
    assert_num!(run("1.0+i"), new_float(1.0, 1.0));
    assert_num!(run("+1.0i"), new_float(0.0, 1.0));
    assert_num!(run("1.0-3.0i"), new_float(1.0, -3.0));
    assert_num!(run("+1.0-3.0i"), new_float(1.0, -3.0));
    assert_num!(run("+1e3+3e3i"), new_float(1e3, 3e3));
    assert_num!(run("+1e-3+3e+3i"), new_float(1e-3, 3e3));
}
*/
