use std::path::PathBuf;
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait
use std::env; // required for print_verbose! macro
use std::panic;


#[macro_use] extern crate krusty_repl;
use krusty_repl::prompt;

#[macro_use] extern crate krusty_core;
use krusty_core::syntax::lexer;
use krusty_core::syntax::parser;
use krusty_core::syntax::evaluator;

const VERSION_STR: &'static str = env!("CARGO_PKG_VERSION");


fn repl_prompt() {
    println!("{} {} {} {}", GREEN!("Welcome to Krusty"), GREEN!(VERSION_STR), "\u{1F980}", GREEN!("repl. Ctrl+C to exit!"));
    let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
    let mut ns = evaluator::NameSpace::new(Some(&cwd), None);

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
                        parser::Phrase::Null => (),
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


fn run_file(filepath: &PathBuf) {
    let mut ns = evaluator::NameSpace::new(Some(filepath), None);
    print_verbose!("Running {:?}", ns.get_path());

    let mut tokens = lexer::lex_file(filepath);
    let tree = parser::parse(&mut tokens);
    let _vo = ns.run(&tree);
    print_verbose!("FINAL\n{:?}\n{:?}", _vo, ns);
}


fn main() {
    let argv: Vec<String> = env::args().collect();
    // println!("{:?}", argv.len());
    if argv.len() > 1 {
        for f in &argv[1..] {
            let filepath = PathBuf::from_slash(f);
            if filepath.is_file() {
                run_file(&filepath);
            }
        }
    } else {
        repl_prompt();
    }
}