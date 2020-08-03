use std::io::Read;
use std::fs::File;
use std::env;



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

        let tokens = lexer::lex(code);
        let tree = parser::parse(tokens);
        // for t in &tree.exprs {
        //     println!("{:?}", t);
        // }
        let mut env = NameSpace::new(None);
        let _vo = env.run(&tree);
        if env::var("VERBOSE").is_ok() {
            println!("FINAL {:?}", env);
        }
        println!("{}", _vo);
    } else {
        panic!("Error: repl is not ready yet!")
    }
}
