use std::collections::HashMap;

#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro

use crate::syntax::{lexer, lexer::Token};
use crate::syntax::{parser, parser::Phrase};
use crate::syntax::evaluator::NameSpace;

use crate::lib::helper;

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
                    Phrase::Object(Token::Text(t)) => {
                        // FIXME: this is a hack to implement unicode newline and tab characters
                        print!("{}", t.replace("\\n", "\u{000A}").replace("\\t", "\u{0009}"))
                    },
                    _ => print!("{}", args[idx]),
                };
            };
            print!("\n");
        },
    };
    Phrase::Null
}


pub fn _type(_: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    func_nargs_eq!(args, 1);
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
    func_nargs_eq!(args, 3);
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
    func_nargs_eq!(args, 1);
    match &args[0] {
        Phrase::Object(Token::Text(p)) => {
            let mut p = ns.get_relative_path(p);
            if !p.ends_with("krt") {
                p.set_extension("krt");
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
    func_nargs_eq!(args, 1);
    match &args[0] {
        Phrase::Object(Token::Text(p)) => {
            let mut p = ns.get_relative_path(&p);
            // let fname = libloading::library_filename(p.file_name().unwrap());
            helper::convert_dylib_os_name(&mut p);

            print_verbose!("import_native({:?})", p);

            let mut new_ns = NameSpace::new(Some(&p), Some(ns));
            new_ns.module.load_dylib();
            new_ns.to_object()
        },
        _ => Phrase::Null
    }
}


pub fn _spill(ns: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    func_nargs_eq!(args, 1);
    match &args[0] {
        Phrase::Mod(m) => {
            ns.module.vars.extend(m.vars.clone());
        },
        _ => ()
    }
    Phrase::Null
}

// ================ iter ================

pub fn _len(_: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    func_nargs_eq!(args, 1);
    let length = match &args[0] {
        Phrase::List(l) => l.len(),
        Phrase::Object(Token::Text(t)) => t.len(),
        _ => panic!("len() not supported")
    };
    Phrase::Object(Token::Number(length as f64))
}

pub fn _foreach(ns: &mut NameSpace, args: &Vec<Phrase>) -> Phrase {
    func_nargs_eq!(args, 2);
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
    func_nargs_le!(args, 1); // 0 or 1 args
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

    helper::load_func(env_native, "print", _print);
    helper::load_func(env_native, "type", _type);
    helper::load_func(env_native, "if", _if);
    helper::load_func(env_native, "len", _len);
    helper::load_func(env_native, "foreach", _foreach);
    helper::load_func(env_native, "vars", _vars);

    helper::load_func(env_native, "import", _import);
    helper::load_func(env_native, "import_native", _import_native);
    helper::load_func(env_native, "spill", _spill);
}
