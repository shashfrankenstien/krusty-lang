use std::collections::HashMap;

#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro

use crate::syntax::{lexer, lexer::Token};
use crate::syntax::{parser, parser::Obj};
use crate::syntax::evaluator::NameSpace;

use crate::lib::loader;

// ================ print =======================


pub fn _print(_: &mut NameSpace, args: &Vec<Obj>) -> Obj {
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


pub fn _type(_: &mut NameSpace, args: &Vec<Obj>) -> Obj {
    if args.len() != 1 {
        panic!("'type' function takes only one argument")
    }
    match args[0] {
        Obj::Object(Token::Text(_)) => Obj::Object(Token::Text("<Text>".to_string())),
        Obj::Object(Token::Number(_)) => Obj::Object(Token::Text("<Number>".to_string())),
        Obj::Func(_) => Obj::Object(Token::Text("<Func>".to_string())),
        Obj::NativeFunc(_) => Obj::Object(Token::Text("<NativeFunc>".to_string())),
        Obj::List(_) => Obj::Object(Token::Text("<List>".to_string())),
        Obj::Bool(_) => Obj::Object(Token::Text("<Bool>".to_string())),
        Obj::Expr(_) => Obj::Object(Token::Text("<Expr>".to_string())),
        Obj::FuncBody(_) => Obj::Object(Token::Text("<FuncBody>".to_string())),
        Obj::Mod(_) => Obj::Object(Token::Text("<Module>".to_string())),
        Obj::Null => Obj::Object(Token::Text("<Null>".to_string())),
        _ => Obj::Object(Token::Text("<Type Not Found>".to_string()))
    }
}

// ================ if =======================

pub fn _if(_: &mut NameSpace, args: &Vec<Obj>) -> Obj {
    if args.len() != 3 {
        panic!("'if' function takes only 3 arguments, {} provided", args.len())
    }
    let condition = match &args[0] {
        Obj::List(l) => l[0].get_bool().unwrap(),
        Obj::Bool(b) => *b,
        _ => panic!("unsupported condition statement")
    };

    let branch = if condition==true { 1 } else { 2 };
    args[branch].clone()
}


// ================ import ================

pub fn _import(ns: &mut NameSpace, args: &Vec<Obj>) -> Obj {
    if args.len() != 1 {
        panic!("can only import one at a time for now")
    }
    match &args[0] {
        Obj::Object(Token::Text(p)) => {
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
        _ => Obj::Null
    }
}



pub fn _import_native(ns: &mut NameSpace, args: &Vec<Obj>) -> Obj {
    if args.len() != 1 {
        panic!("can only import one at a time for now")
    }
    match &args[0] {
        Obj::Object(Token::Text(p)) => {
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
        _ => Obj::Null
    }
}


// ================ iter ================

pub fn _len(_: &mut NameSpace, args: &Vec<Obj>) -> Obj {
    if args.len() != 1 {
        panic!("can only import one at a time for now")
    }
    let length = match &args[0] {
        Obj::List(l) => l.len(),
        Obj::Object(Token::Text(t)) => t.len(),
        _ => panic!("len() not supported")
    };
    Obj::Object(Token::Number(length as f64))
}

pub fn _foreach(ns: &mut NameSpace, args: &Vec<Obj>) -> Obj {
    if args.len() != 2 {
        panic!("illegal number of arguments. Expected 2")
    }
    if let Obj::Func(_) | Obj::NativeFunc(_) = &args[1] {
        let res: Vec<Obj>;
        return match &args[0] {
            Obj::List(l) => {
                res = l.iter().map(|x| ns.eval_func_obj(&args[1], &x, None)).collect();
                Obj::List(res)
            },
            Obj::Object(Token::Text(t)) => {
                res = t.chars().map(|c| ns.eval_func_obj(&args[1], &Obj::Object(Token::Text(c.to_string())), None)).collect();
                Obj::List(res)
            },
            _ => panic!("iteration not supported")
        }
    } else {
        panic!("second argument should be a function");
    }
}

// ================ module inspect ================

pub fn _vars(ns: &mut NameSpace, args: &Vec<Obj>) -> Obj {
    if args.len() > 1 {
        panic!("illegal number of arguments. Expected 0 or 1")
    }
    let mut vars: Vec<Obj> = Vec::new();
    if args.len() == 0 {
        for (k,_) in &ns.module.vars {
            vars.push(Obj::Object(Token::Text(k.clone())));
        }
        Obj::List(vars)
    }
    else if let Obj::Mod(m) = &args[0] {
        for (k,_) in &m.vars {
            vars.push(Obj::Object(Token::Text(k.clone())));
        }
        Obj::List(vars)
    } else {
        Obj::Null
    }
}


// ================ namespace helper functions ====================


pub fn load(env_native: &mut HashMap<String, Obj>) {
    env_native.insert("null".to_string(), Obj::Null);
    env_native.insert("true".to_string(), Obj::Bool(true));
    env_native.insert("false".to_string(), Obj::Bool(false));

    loader::load_func(env_native, "print", _print);
    loader::load_func(env_native, "type", _type);
    loader::load_func(env_native, "if", _if);
    loader::load_func(env_native, "len", _len);
    loader::load_func(env_native, "foreach", _foreach);
    loader::load_func(env_native, "vars", _vars);

    loader::load_func(env_native, "import", _import);
    loader::load_func(env_native, "import_native", _import_native);

}
