use std::path::PathBuf;
use std::env; // required for print_verbose! macro
use std::process;

extern crate ctrlc;

#[macro_use]
pub mod macros;

mod syntax {
    pub mod lexer;
    pub mod parser;
    pub mod evaluator;
}

mod lib {
    pub mod builtins;
}

#[macro_use]
mod repl {
    #[macro_use]
    pub mod colors;
    pub mod prompt;
}

use syntax::lexer;
use syntax::parser;
use syntax::evaluator::NameSpace;
use repl::prompt;


fn repl_prompt(ns: &mut NameSpace) {
    println!("{} {} {}", GREEN!("Welcome to Krusty"), "\u{1F980}", GREEN!("repl. Ctrl+C to exit!"));
    let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
    ns.set_path(&cwd);

    ctrlc::set_handler(move || {
        println!("KeyboardInterrupt");
        process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    let mut i = 0;
    loop {
        let buffer = prompt::prompt(i);
        match buffer {
            None => (),
            Some(buf) => {
                let mut tokens = lexer::lex(&buf);
                let parsed = parser::parse(&mut tokens);
                let out = ns.run(&parsed);
                match out {
                    parser::Obj::Null => (),
                    _ => println!("{}", ns.resolve(&out))
                }
            }
        }
        i += 1;
    }
}


fn run_file(filepath: &PathBuf, ns: &mut NameSpace) -> parser::Obj {
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
        let filepath = PathBuf::from(&argv[1]);
        if filepath.is_file() {
            let mut ns = NameSpace::new(None);
            let _vo = run_file(&filepath, &mut ns);
            print_verbose!("FINAL\n{:?}\n{:?}", _vo, ns);
        }
    } else {
        let mut ns = NameSpace::new(None);
        repl_prompt(&mut ns);
    }
}
