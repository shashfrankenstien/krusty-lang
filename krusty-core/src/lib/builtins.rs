use std::path::PathBuf;
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait
use std::collections::HashMap;

#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro
use libloading;

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
            let cur_path = ns.get_path();
            let p = match cur_path {
                Some(pbuf) => {
                    let mut newbuf = pbuf.clone();
                    if newbuf.is_dir() {
                        newbuf.push(PathBuf::from_slash(p)); // push new filename
                    } else {
                        newbuf.set_file_name(PathBuf::from_slash(p)); // replace filename
                    }
                    if !newbuf.ends_with("kry") {
                        newbuf.set_extension("kry");
                    }
                    newbuf
                },
                None => PathBuf::from_slash(p)
            };
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


fn call_dynamic_loader(path: &str, hm: &mut HashMap<String, Obj>) -> Result<u32, Box<dyn std::error::Error>> {
    let lib = libloading::Library::new(path)?;
    unsafe {
        let func: libloading::Symbol<unsafe extern fn(&mut HashMap<String, Obj>)> = lib.get(b"load")?;
        func(hm);
        Ok(1)
    }
}

pub fn _import_native(ns: &mut NameSpace, args: &Vec<Obj>) -> Obj {
    if args.len() != 1 {
        panic!("can only import one at a time for now")
    }
    match &args[0] {
        Obj::Object(Token::Text(p)) => {
            let cur_path = ns.get_path();
            let p = match cur_path {
                Some(pbuf) => {
                    let mut newbuf = pbuf.clone();
                    if newbuf.is_dir() {
                        newbuf.push(PathBuf::from_slash(p)); // push new filename
                    } else {
                        newbuf.set_file_name(PathBuf::from_slash(p)); // replace filename
                    }
                    if !newbuf.ends_with("so") {
                        newbuf.set_extension("so");
                    }
                    newbuf
                },
                None => PathBuf::from_slash(p)
            };
            print_verbose!("import_native({:?})", p);
            // let mut tokens = lexer::lex_file(&p);
            // let tree = parser::parse(&mut tokens);

            let mut new_ns = NameSpace::new(Some(&p), Some(ns));
            match call_dynamic_loader(p.to_str().unwrap(), &mut new_ns.module.vars) {
                Ok(_) => (),
                Err(e) => {
                    println!("{:?}", e);
                    panic!("error loading module")
                }
            }
            // new_ns.run(&tree);
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
