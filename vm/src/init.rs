use {assemble, ASM, Environment, Register, Value, VM};

pub fn init_env<T>() -> Environment<T> {
    let env = Environment::new();

    let add = vec![ASM::Add(Register(0), Register(1), Register(2))];
    add_primitive(&env, "+".to_string(), add);

    let sub = vec![ASM::Sub(Register(0), Register(1), Register(2))];
    add_primitive(&env, "-".to_string(), sub);

    let mul = vec![ASM::Mul(Register(0), Register(1), Register(2))];
    add_primitive(&env, "*".to_string(), mul);

    let eq = vec![ASM::Eq(Register(0), Register(1), Register(2))];
    add_primitive(&env, "=".to_string(), eq);

    let lt = vec![ASM::LT(Register(0), Register(1), Register(2))];
    add_primitive(&env, "<".to_string(), lt);

    let cons = vec![ASM::Cons(Register(0), Register(1), Register(2))];
    add_primitive(&env, "cons".to_string(), cons);
    let car = vec![ASM::Car(Register(0), Register(1))];
    add_primitive(&env, "car".to_string(), car);
    let cdr = vec![ASM::Cdr(Register(0), Register(1))];
    add_primitive(&env, "cdr".to_string(), cdr);

    env.define_variable(VM::<T>::intern_symbol("pi".to_string()), Value::Float(std::f64::consts::PI));
    env.define_variable(VM::<T>::intern_symbol("e".to_string()), Value::Float(std::f64::consts::E));

    env
}

fn add_primitive<T>(env: &Environment<T>, name: String, code: Vec<ASM<T>>) {
    let (code, consts) = assemble(code);
    env.define_variable(VM::<T>::intern_symbol(name), Value::Lambda(env.clone(), code, consts));
}
