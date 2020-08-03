use std::fmt;
use std::collections::HashMap;
use crate::syntax::lexer::Token;
use crate::syntax::parser::{Obj, ExprList};


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Text(t) => write!(f, "\"{}\"", t),
            _ => write!(f, "{:?}", self),
        }
    }
}


impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::Object(o) => write!(f, "{}", o),
            Obj::Operator(op) => write!(f, "{}", op),
            Obj::List(l) => {
                write!(f, "(").unwrap();
                for member in l {
                    write!(f, "{},", member).unwrap();
                };
                write!(f, ")")
            },
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for ExprList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExprList({:?})", self.exprs)
    }
}


fn _print_iterable(l: &Vec<Obj>) {
    print!("(");
    for member in l.iter() {
        print!("{},", member);
    };
    println!(")");
}

pub fn print(args: &Vec<Obj>) -> Obj {
    match args.len() {
        0 => println!("Null"),
        1 => println!("{}", args[0]),
        _ => _print_iterable(args)
    };
    Obj::Null
}


pub fn load(env_bi: &mut HashMap<String, Obj>) {
    env_bi.insert("print".to_string(), Obj::BuiltinFunc("print".to_string()));
}

pub fn find(name: &str) -> fn(&Vec<Obj>) -> Obj {
    match name {
        "print" => print,
        _ => panic!("'{}' not found", name)
    }
}
