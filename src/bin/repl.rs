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

use std::borrow::Cow;
use std::fs::File;
use std::io::Read;

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

    if let Ok(mut f) = File::open("~/.config/minerva/init.ss") {
        let mut input = String::new();
        f.read_to_string(&mut input).unwrap();
        run(&mut vm, None, input);
    }

    let config = config::Builder::new()
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .auto_add_history(true)
        .build();
    let mut rl: Editor<Repl<()>> = Editor::with_config(config);
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


    #[cfg(feature="profile")]
    flame::dump_html(&mut std::fs::File::create("profile2.html").unwrap()).unwrap();
}

fn run<T>(vm: &mut VM<T>, env: Option<&Environment<T>>, input: String) {
    let tokens = match minerva::Tokenizer::tokenize(&input) {
        Ok(t) => t,
        Err(e) => {
            println!("ERROR: {}", e);
            return;
        }
    };

    let ast: Vec<minerva::Ast<T>> = match minerva::Parser::parse(tokens) {
        Ok(o) => o,
        Err(e) => {
            println!("ERROR: {}", e);
            return;
        }
    };

    for ast in ast {
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

fn swap_cash_vars<T>(env: &Environment<T>, v: Value<T>) {
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

struct Repl<T> {
    env: Environment<T>,
    path: FilenameCompleter,
    keywords: Vec<String>,
    m: MatchingBracketHighlighter,
}

impl<T> Repl<T> {
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

impl<T> Completer for Repl<T> {
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

impl<T> Validator for Repl<T> {
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

impl<T> Helper for Repl<T> {
}

impl<T> Hinter for Repl<T> {
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl<T> Highlighter for Repl<T> {
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
