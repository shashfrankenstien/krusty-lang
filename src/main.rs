use std::io::Read;
use std::fs::File;
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


fn main() {
    // Prints each argument on a separate line
    let argv: Vec<String> = env::args().collect();
    // println!("{:?}", argv.len());
    if argv.len() > 1 {
        let filename = &argv[1];

        let mut f = File::open(filename).expect("Oh, no such file!");
        let mut code = String::new();
        f.read_to_string(&mut code).expect("Can't read this");

        let mut tokens = lexer::lex(code);
        let tree = parser::parse(&mut tokens);
        // for t in &tree {
        //     println!("{:?}", t);
        // }
        let mut environment = NameSpace::new(None);
        let _vo = environment.run(&tree);
        print_verbose!("FINAL {:?}", environment);
        // println!("{}", _vo);
    } else {
        panic!("Error: repl is not ready yet!")
    }
}
