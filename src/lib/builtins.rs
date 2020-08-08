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



pub fn print(args: &Vec<Obj>) -> Obj {
    match args.len() {
        0 => println!(""),
        _ => {
            for idx in 0..args.len() {
                if idx > 0 {
                    print!(" ");
                }
                match &args[idx] {
                    Obj::Object(Token::Number(n)) => print!("{}", n),
                    Obj::Object(Token::Text(t)) => print!("{}", t),
                    _ => print!("{}", args[idx]),
                };
            };
            print!("\n");
        },
    };
    Obj::Null
}


pub fn load(env_bi: &mut HashMap<String, Obj>) {
    env_bi.insert("print".to_string(), Obj::BuiltinFunc("print".to_string()));
    env_bi.insert("null".to_string(), Obj::Null);
}

pub fn find_func(name: &str) -> fn(&Vec<Obj>) -> Obj {
    match name {
        "print" => print,
        _ => panic!("'{}' not found", name)
    }
}
