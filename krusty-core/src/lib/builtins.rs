use std::collections::HashMap;

#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro

use crate::syntax::{lexer, lexer::Token};
use crate::syntax::{parser, parser::Phrase};
use crate::syntax::evaluator::NameSpace;

use crate::lib::loader;

// ================ print =======================


pub fn _print(_: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    match args.len() {
        0 => println!(""),
        _ => {
            for idx in 0..args.len() {
                if idx > 0 {
                    print!(" ");
                }
                match &args[idx] {
                    Phrase::Object(Token::Number(n)) => print!("{}", n),
                    Phrase::Object(Token::Text(t)) => print!("{}", t),
                    _ => print!("{}", args[idx]),
                };
            };
            print!("\n");
        },
    };
    Phrase::Null
}


pub fn _type(_: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    if args.len() != 1 {
        panic!("'type' function takes only one argument")
    }
    match args[0] {
        Phrase::Object(Token::Text(_)) => Phrase::Object(Token::Text("<Text>".to_string())),
        Phrase::Object(Token::Number(_)) => Phrase::Object(Token::Text("<Number>".to_string())),
        Phrase::Func(_) => Phrase::Object(Token::Text("<Func>".to_string())),
        Phrase::NativeFunc(_) => Phrase::Object(Token::Text("<NativeFunc>".to_string())),
        Phrase::List(_) => Phrase::Object(Token::Text("<List>".to_string())),
        Phrase::Bool(_) => Phrase::Object(Token::Text("<Bool>".to_string())),
        Phrase::Expr(_) => Phrase::Object(Token::Text("<Expr>".to_string())),
        Phrase::FuncBody(_) => Phrase::Object(Token::Text("<FuncBody>".to_string())),
        Phrase::Mod(_) => Phrase::Object(Token::Text("<Module>".to_string())),
        Phrase::Null => Phrase::Object(Token::Text("<Null>".to_string())),
        _ => Phrase::Object(Token::Text("<Type Not Found>".to_string()))
    }
}

// ================ if =======================

pub fn _if(_: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    if args.len() != 3 {
        panic!("'if' function takes only 3 arguments, {} provided", args.len())
    }
    let condition = match &args[0] {
        Phrase::List(l) => l[0].get_bool().unwrap(),
        Phrase::Bool(b) => *b,
        _ => panic!("unsupported condition statement")
    };

    let branch = if condition==true { 1 } else { 2 };
    args[branch].clone()
}


// ================ import ================

pub fn _import(ns: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    if args.len() != 1 {
        panic!("can only import one at a time for now")
    }
    match &args[0] {
        Phrase::Object(Token::Text(p)) => {
            let mut p = ns.get_relative_path(p);
            if !p.ends_with("kry") {
                p.set_extension("kry");
            }
            print_verbose!("import({:?})", p);
            let mut tokens = lexer::lex_file(&p);
            let tree = parser::parse(&mut tokens);

            let mut new_ns = NameSpace::new(Some(&p), Some(ns));
            new_ns.run(&tree);
            new_ns.to_object()
        },
        _ => Phrase::Null
    }
}



pub fn _import_native(ns: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    if args.len() != 1 {
        panic!("can only import one at a time for now")
    }
    match &args[0] {
        Phrase::Object(Token::Text(p)) => {
            let mut p = ns.get_relative_path(&p);
            // let fname = libloading::library_filename(p.file_name().unwrap());
            #[cfg(target_os = "windows")]
            if !p.ends_with("dll") {
                p.set_extension("dll");
            }
            #[cfg(target_os = "macos")]
            if !p.ends_with("dylib") {
                p.set_extension("dylib");
            }

            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
            if !p.ends_with("so") {
                p.set_extension("so");
            }

            print_verbose!("import_native({:?})", p);

            let mut new_ns = NameSpace::new(Some(&p), Some(ns));
            new_ns.module.load_dylib();
            new_ns.to_object()
        },
        _ => Phrase::Null
    }
}


// ================ iter ================

pub fn _len(_: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    if args.len() != 1 {
        panic!("can only import one at a time for now")
    }
    let length = match &args[0] {
        Phrase::List(l) => l.len(),
        Phrase::Object(Token::Text(t)) => t.len(),
        _ => panic!("len() not supported")
    };
    Phrase::Object(Token::Number(length as f64))
}

pub fn _foreach(ns: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    if args.len() != 2 {
        panic!("illegal number of arguments. Expected 2")
    }
    if let Phrase::Func(_) | Phrase::NativeFunc(_) = &args[1] {
        let res: Vec<Phrase>;
        return match &args[0] {
            Phrase::List(l) => {
                res = l.iter().map(|x| ns.eval_func_obj(&args[1], &x, None)).collect();
                Phrase::List(res)
            },
            Phrase::Object(Token::Text(t)) => {
                res = t.chars().map(|c| ns.eval_func_obj(&args[1], &Phrase::Object(Token::Text(c.to_string())), None)).collect();
                Phrase::List(res)
            },
            _ => panic!("iteration not supported")
        }
    } else {
        panic!("second argument should be a function");
    }
}

// ================ module inspect ================

pub fn _vars(ns: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    if args.len() > 1 {
        panic!("illegal number of arguments. Expected 0 or 1")
    }
    let mut vars: Vec<Phrase> = Vec::new();
    if args.len() == 0 {
        for (k,_) in &ns.module.vars {
            vars.push(Phrase::Object(Token::Text(k.clone())));
        }
        Phrase::List(vars)
    }
    else if let Phrase::Mod(m) = &args[0] {
        for (k,_) in &m.vars {
            vars.push(Phrase::Object(Token::Text(k.clone())));
        }
        Phrase::List(vars)
    } else {
        Phrase::Null
    }
}


// ================ namespace helper functions ====================


pub fn load(env_native: &mut HashMap<String, Phrase>) {
    env_native.insert("null".to_string(), Phrase::Null);
    env_native.insert("true".to_string(), Phrase::Bool(true));
    env_native.insert("false".to_string(), Phrase::Bool(false));

    loader::load_func(env_native, "print", _print);
    loader::load_func(env_native, "type", _type);
    loader::load_func(env_native, "if", _if);
    loader::load_func(env_native, "len", _len);
    loader::load_func(env_native, "foreach", _foreach);
    loader::load_func(env_native, "vars", _vars);

    loader::load_func(env_native, "import", _import);
    loader::load_func(env_native, "import_native", _import_native);

}
