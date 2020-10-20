use std::io::Read; // for read_to_string
use std::fs;
use std::path::PathBuf;
use std::env; // required for print_verbose! macro

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
    let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
    ns.set_path(&cwd);
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
                    _ => println!("{}", out)
                }
            }
        }
        i += 1;
    }
}


fn run_file(filepath: &PathBuf, ns: &mut NameSpace) -> parser::Obj {
    ns.set_path(&filepath);
    print_verbose!("Running {:?}", ns.get_path());

    let mut f = fs::File::open(filepath).expect("Oh, no such file!");
    let mut code = String::new();
    f.read_to_string(&mut code).expect("Can't read this");

    let mut tokens = lexer::lex(&code);
    let tree = parser::parse(&mut tokens);
    ns.run(&tree)
}


fn main() {
    // Prints each argument on a separate line
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
