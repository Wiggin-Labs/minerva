extern crate minerva;
extern crate rustyline;
extern crate string_interner;
extern crate vm;

use minerva::{ParseError, Token};
use vm::{assemble, init_env, Environment, Operation, Register, Value, VM};

use rustyline::{Context, Editor, Helper};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::{self, CompletionType, EditMode};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{Validator, ValidationResult, ValidationContext};
use string_interner::{get_symbol, get_value};

use std::fs;
use std::borrow::Cow;

fn main() {
    let mut vm = VM::new();
    //vm.set_debug();
    let env = init_env();
    vm.assign_environment(env.clone());
    let repl = Repl {
        env: env.clone(),
        keywords: vec!["define".into(), "if".into(), "lambda".into(), "begin".into()],
        path: FilenameCompleter::new(),
        m: MatchingBracketHighlighter::new(),
    };

    if let Ok(input) = fs::read_to_string("~/.config/minerva/init.ss") {
        run(&mut vm, None, input);
    }

    let config = config::Builder::new()
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .auto_add_history(true)
        .build();
    let mut rl: Editor<Repl> = Editor::with_config(config);
    rl.set_helper(Some(repl));

    let mut ctrlc = false;
    loop {
        let s = get_symbol("$PROMPT".into());
        let prompt = if let Some(v) = env.lookup_variable_value(s) {
            vm.assign_register(Register(0), v);
            vm.load_code(vec![Operation::Call(Register(0))], vec![]);
            vm.run();
            let p = vm.load_register(Register(0));
            if p.is_string() {
                let v = p.to_string();
                let s = v.str.clone();
                Box::into_raw(v);
                s
            } else {
                println!("ERROR: Expected $PROMPT to produce a string!");
                ">> ".to_string()
            }
        } else {
            ">> ".to_string()
        };

        let input = match rl.readline(&prompt) {
            Ok(i) => i,
            Err(e) => match e {
                ReadlineError::Eof => return,
                ReadlineError::Interrupted => if ctrlc {
                    return;
                } else {
                    ctrlc = true;
                    continue;
                },
                _ => panic!("{}", e),
            }
        };
        ctrlc = false;

        if "exit\n" == input {
            break;
        }

        run(&mut vm, Some(&env), input);
    }
}

fn run(vm: &mut VM, env: Option<&Environment>, input: String) {
    let tokens = match minerva::Tokenizer::tokenize(&input) {
        Ok(t) => t,
        Err(e) => {
            println!("ERROR: {}", e);
            return;
        }
    };

    let ast: Vec<minerva::Ast> = match minerva::Parser::parse(tokens) {
        Ok(o) => o,
        Err(e) => {
            println!("ERROR: {}", e);
            return;
        }
    };

    for mut ast in ast {
        threading(&mut ast);
        let ir = minerva::compile(ast);
        let ir = minerva::optimize(ir);
        println!("IR:");
        for i in &ir {
            println!("{}", i);
        }
        println!();

        println!("ASM:");
        let asm = minerva::output_asm(ir);
        for i in &asm {
            println!("{}", i);
        }
        println!();

        println!("RESULT:");
        let (code, consts) = assemble(asm);
        vm.load_code(code, consts);
        vm.run();
        let result = vm.load_register(Register(0));
        if !result.is_void() {
            println!("{}", result);
            if let Some(env) = env {
                swap_cash_vars(env, result);
            }
        }
    }
}

fn threading(ast: &mut minerva::Ast) {
    use minerva::Ast::*;

    match ast {
        Define { value, .. } => threading(&mut *value),
        Lambda { body, .. } => for a in body {
            threading(a);
        },
        If { predicate, consequent, alternative } => {
            threading(&mut *predicate);
            threading(&mut *consequent);
            threading(&mut *alternative);
        }
        Begin(v) => for a in v {
            threading(a);
        },
        Apply(v) => match v[0] {
            Ident(s) => if "->" == get_value(s).unwrap() {
                let mut v1 = v.clone();
                v1.remove(0);
                *v = handle_threading(v1);
            } else {
                for a in  v {
                    threading(a);
                }
            }
            _ => for a in  v {
                threading(a);
            },
        },
        _ => (),
    }
}

fn handle_threading(mut ast: Vec<minerva::Ast>) -> Vec<minerva::Ast> {
    use minerva::Ast::*;
    if ast.len() == 0 {
        return vec![];
    }

    let mut prev = ast.remove(0);
    for mut a in ast {
        match a {
            Apply(mut v) => {
                let mut done = false;
                for s in v.iter_mut() {
                    match s {
                        Ident(sym) => {
                            if "_" == get_value(*sym).unwrap() {
                                *s = prev.clone();
                                done = true;
                            } else {
                                threading(s);
                            }
                        }
                        _ => threading(s),
                    }
                }
                if !done {
                    v.push(prev);
                }
                prev = Apply(v);
            }
            _ => {
                threading(&mut a);
                prev = Apply(vec![a, prev]);
            }
        }
    }

    if let Apply(v) = prev {
        v
    } else {
        // TODO
        let x = get_symbol("x".to_string());
        let identity = Lambda {
            args: vec![x],
            body: vec![Ident(x)],
        };
        vec![identity, prev]
    }
}

fn swap_cash_vars(env: &Environment, v: Value) {
    let cash1 = get_symbol("$1".into());
    let cash2 = get_symbol("$2".into());
    let cash3 = get_symbol("$3".into());
    let cash4 = get_symbol("$4".into());
    let cash5 = get_symbol("$5".into());
    let cash6 = get_symbol("$6".into());
    let cash7 = get_symbol("$7".into());
    let cash8 = get_symbol("$8".into());
    let cash9 = get_symbol("$9".into());
    if let Some(v) = env.lookup_variable_value(cash8) {
        env.define_variable(cash9, v);
    }
    if let Some(v) = env.lookup_variable_value(cash7) {
        env.define_variable(cash8, v);
    }
    if let Some(v) = env.lookup_variable_value(cash6) {
        env.define_variable(cash7, v);
    }
    if let Some(v) = env.lookup_variable_value(cash5) {
        env.define_variable(cash6, v);
    }
    if let Some(v) = env.lookup_variable_value(cash4) {
        env.define_variable(cash5, v);
    }
    if let Some(v) = env.lookup_variable_value(cash3) {
        env.define_variable(cash4, v);
    }
    if let Some(v) = env.lookup_variable_value(cash2) {
        env.define_variable(cash3, v);
    }
    if let Some(v) = env.lookup_variable_value(cash1) {
        env.define_variable(cash2, v);
    }
    env.define_variable(cash1, v);
}

struct Repl {
    env: Environment,
    path: FilenameCompleter,
    keywords: Vec<String>,
    m: MatchingBracketHighlighter,
}

impl Repl {
    fn get_defs(&self, start: &str) -> Vec<String> {
        let mut keywords: Vec<String> = self.keywords.iter()
            .filter(|s| s.starts_with(start))
            .map(|s| s.clone())
            .collect();
        let mut defs = self.env.get_definitions().iter()
            .map(|s| get_value(*s).unwrap())
            .filter(|s| s.starts_with(start))
            .collect();
        keywords.append(&mut defs);
        keywords
    }
}

impl Completer for Repl {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let l = &line[0..pos];
        match minerva::Tokenizer::tokenize(l) {
            Ok(tokens) => {
                let (p, candidates) = if let Some(Token::Symbol(s)) = tokens.last() {
                    if l.chars().last().unwrap().is_whitespace() {
                        (pos, self.get_defs(""))
                    } else {
                        let s = get_value(*s).unwrap();
                        (pos - s.len(), self.get_defs(&s))
                    }
                } else {
                    (pos, self.get_defs(""))
                };
                let candidates = candidates.into_iter().map(|s| Pair { display: s.clone(), replacement: s }).collect();
                Ok((p, candidates))
            },
            Err(ParseError::InString) => return self.path.complete_path(line, pos),
            // TODO
            _ => unreachable!(),
        }

    }
}

impl Validator for Repl {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> Result<ValidationResult, ReadlineError> {
        match minerva::Tokenizer::tokenize(ctx.input()) {
            Ok(tokens) => if tokens.is_empty() ||
                tokens.iter().filter(|&t| t.is_left_paren()).count()
                > tokens.iter().filter(|&t| t.is_right_paren()).count()
            {
                return Ok(ValidationResult::Incomplete);
            },
            Err(ParseError::InString) => return Ok(ValidationResult::Incomplete),
            _ => (),
        }

        Ok(ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl Helper for Repl {
}

impl Hinter for Repl {
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for Repl {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.m.highlight(line, pos)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, default: bool) -> Cow<'b, str> {
        self.m.highlight_prompt(prompt, default)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        self.m.highlight_hint(hint)
    }

    fn highlight_candidate<'c>(&self, candidate: &'c str, completion: CompletionType) -> Cow<'c, str> {
        self.m.highlight_candidate(candidate, completion)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.m.highlight_char(line, pos)
    }
}
