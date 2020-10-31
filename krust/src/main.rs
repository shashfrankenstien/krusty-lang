use std::path::PathBuf;
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait
use std::env; // required for print_verbose! macro
use std::panic;

#[macro_use]
mod repl {
    #[macro_use]
    pub mod colors;
    pub mod prompt;
}

use repl::prompt;

#[macro_use] extern crate krusty_core;
use krusty_core::syntax::lexer;
use krusty_core::syntax::parser;
use krusty_core::syntax::evaluator;

const VERSION_STR: &'static str = env!("CARGO_PKG_VERSION");


fn repl_prompt(ns: &mut evaluator::NameSpace) {
    println!("{} {} {} {}", GREEN!("Welcome to Krusty"), GREEN!(VERSION_STR), "\u{1F980}", GREEN!("repl. Ctrl+C to exit!"));
    let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
    ns.set_path(&cwd);

    let mut cli = prompt::Prompt::new();
    loop {
        let buffer = cli.read_expr();
        match buffer {
            Ok(buf) if buf.trim().len() == 0 => (),
            Ok(buf) => {
                let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    let mut tokens = lexer::lex(&buf);
                    let parsed = parser::parse(&mut tokens);
                    let out = ns.run(&parsed);
                    match out {
                        parser::Obj::Null => (),
                        _ => println!("{}", ns.resolve(&out))
                    }
                }));
                if result.is_err() {
                    println!("{}", RED!("Error in expression!"));
                }
            },
            Err(e) => { // ctrl-c or ctrl-d
                println!("{}", RED!(e.to_string()));
                break;
            }
        }
    }
}


fn run_file(filepath: &PathBuf, ns: &mut evaluator::NameSpace) -> parser::Obj {
    ns.set_path(&filepath);
    print_verbose!("Running {:?}", ns.get_path());

    let mut tokens = lexer::lex_file(filepath);
    let tree = parser::parse(&mut tokens);
    ns.run(&tree)
}


fn main() {
    let argv: Vec<String> = env::args().collect();
    // println!("{:?}", argv.len());
    if argv.len() > 1 {
        let filepath = PathBuf::from_slash(&argv[1]);
        if filepath.is_file() {
            let mut ns = evaluator::NameSpace::new(None);
            let _vo = run_file(&filepath, &mut ns);
            print_verbose!("FINAL\n{:?}\n{:?}", _vo, ns);
        }
    } else {
        let mut ns = evaluator::NameSpace::new(None);
        repl_prompt(&mut ns);
    }
}
