use std::io::Read;
use std::fs::File;
use std::env;



mod syntax {
    pub mod lexer;
    pub mod parser;
    pub mod evaluator;
}

use syntax::parser;
use syntax::lexer;
use syntax::evaluator::Env;



fn main() {
    // Prints each argument on a separate line
    let argv: Vec<String> = env::args().collect();
    println!("{:?}", argv.len());
    if argv.len() > 1 {
        let filename = &argv[1];

        let mut f = File::open(filename).expect("Oh, no such file!");
        let mut code = String::new();
        f.read_to_string(&mut code).expect("Can't read this");

        let tokens = lexer::lex(code);
        // println!("{:?}", t)
        let tree = parser::parse(tokens);

        let mut env = Env::new(None);
        let _vo = env.disperse(&tree);
    }
}
