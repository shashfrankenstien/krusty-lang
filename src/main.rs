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

use syntax::lexer;
use syntax::parser;
use syntax::evaluator::NameSpace;


fn run_file(filepath: &PathBuf, ns: &mut NameSpace) -> parser::Obj {
    ns.set_path(&filepath);
    print_verbose!("Running {:?}", ns.get_path());

    let mut f = fs::File::open(filepath).expect("Oh, no such file!");
    let mut code = String::new();
    f.read_to_string(&mut code).expect("Can't read this");

    let mut tokens = lexer::lex(code);
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
        panic!("Error: repl is not ready yet!")
    }
}
