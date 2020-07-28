use std::io::Read;
use std::fs::File;

mod syntax {
    pub mod lexer;
    pub mod parser;
}


fn main() {
    let mut f = File::open("test.mon").expect("Oh, no such file!");
    let mut code = String::new();
    f.read_to_string(&mut code).expect("Can't read this");

    let t = syntax::lexer::lex(code);
    // println!("{:?}", t)
    syntax::parser::parse(t);
}
