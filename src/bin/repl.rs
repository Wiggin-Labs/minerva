extern crate akuma;
#[cfg(feature="profile")]
extern crate flame;
extern crate rustyline;
extern crate string_interner;
extern crate vm;

use akuma::{ParseError, Token};
use vm::{assemble, init_env, Environment, Register, VM};

use rustyline::{Context, Editor, Helper};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::{self, CompletionType, EditMode};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{Validator, ValidationResult, ValidationContext};
use string_interner::INTERNER;

use std::borrow::Cow;

fn main() {
    let mut vm = VM::new();
    //vm.set_debug();
    let env = init_env();
    vm.assign_environment(env.clone());
    let repl = Repl {
        env: env,
        keywords: vec!["define".into(), "if".into(), "lambda".into(), "begin".into()],
        path: FilenameCompleter::new(),
        m: MatchingBracketHighlighter::new(),
    };

    let config = config::Builder::new()
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .auto_add_history(true)
        .build();
    let mut rl: Editor<Repl> = Editor::with_config(config);
    rl.set_helper(Some(repl));

    let mut ctrlc = false;
    loop {
        let input = match rl.readline(">> ") {
            Ok(i) => i,
            Err(e) => match e {
                ReadlineError::Eof => return,
                ReadlineError::Interrupted => if ctrlc {
                    return;
                } else {
                    ctrlc = true;
                    continue;
                },
                _ => panic!(e),
            }
        };
        ctrlc = false;

        if "exit\n" == input {
            break;
        }

        let tokens = match akuma::Tokenizer::tokenize(&input) {
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
            for i in &asm {
                println!("{}", i);
            }
            vm.load_code(assemble(asm));
            vm.run();
            let result = vm.load_register(Register(0));
            if !result.is_void() {
                println!("{}", result);
            }
        }
    }


    #[cfg(feature="profile")]
    flame::dump_html(&mut std::fs::File::create("profile2.html").unwrap()).unwrap();
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
            .map(|s| INTERNER.lock().unwrap().get_value(*s).unwrap())
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
        match akuma::Tokenizer::tokenize(l) {
            Ok(tokens) => {
                let (p, candidates) = if let Some(Token::Symbol(s)) = tokens.last() {
                    if l.chars().last().unwrap().is_whitespace() {
                        (pos, self.get_defs(""))
                    } else {
                        let s = INTERNER.lock().unwrap().get_value(*s).unwrap();
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
        match akuma::Tokenizer::tokenize(ctx.input()) {
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
