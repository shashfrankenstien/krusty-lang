use std::fmt;
use std::collections::HashMap;
use crate::syntax::lexer::Token;
use crate::syntax::parser::Obj;


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
            Obj::Bool(b) => write!(f, "{}", b),
            Obj::Null => write!(f, "null"),
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

// impl fmt::Display for ExprList {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "ExprList({:?})", self.exprs)
//     }
// }



pub fn _print(args: &Vec<Obj>) -> Obj {
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


pub fn _type(args: &Vec<Obj>) -> Obj {
    if args.len() != 1 {
        panic!("'type' function takes only one argument")
    }
    match args[0] {
        Obj::Object(Token::Text(_)) => Obj::Object(Token::Text("<Text>".to_string())),
        Obj::Object(Token::Number(_)) => Obj::Object(Token::Text("<Number>".to_string())),
        Obj::Func(_) => Obj::Object(Token::Text("<Func>".to_string())),
        Obj::BuiltinFunc(_) => Obj::Object(Token::Text("<BuiltinFunc>".to_string())),
        Obj::List(_) => Obj::Object(Token::Text("<List>".to_string())),
        Obj::Bool(_) => Obj::Object(Token::Text("<Bool>".to_string())),
        Obj::Expr(_) => Obj::Object(Token::Text("<Expr>".to_string())),
        Obj::Group(_) => Obj::Object(Token::Text("<ExprGroup>".to_string())),
        Obj::Null => Obj::Object(Token::Text("<Null>".to_string())),
        _ => Obj::Object(Token::Text("<Type Not Found>".to_string()))
    }
}


pub fn load(env_bi: &mut HashMap<String, Obj>) {
    env_bi.insert("null".to_string(), Obj::Null);
    env_bi.insert("true".to_string(), Obj::Bool(true));
    env_bi.insert("false".to_string(), Obj::Bool(false));
    env_bi.insert("print".to_string(), Obj::BuiltinFunc("print".to_string()));
    env_bi.insert("type".to_string(), Obj::BuiltinFunc("type".to_string()));
}

pub fn find_func(name: &str) -> fn(&Vec<Obj>) -> Obj {
    match name {
        "print" => _print,
        "type" => _type,
        _ => panic!("'{}' not found", name)
    }
}
