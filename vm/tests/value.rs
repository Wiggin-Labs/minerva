extern crate vm;

use vm::*;

#[test]
fn string() {
    let s = String::from("abc");
    let v: Value<()> = Value::String(s);
    let s = v.to_string();
    assert_eq!(&s.str, "abc");
}
