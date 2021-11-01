use std::collections::HashMap;

#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro

use crate::syntax::{lexer, lexer::Token};
use crate::syntax::{parser, parser::Block};
use crate::syntax::evaluator::NameSpace;

use super::errors::{Error, KrustyErrorType};
use super::helper;

// ================ print =======================


fn _print(_: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    match args.len() {
        0 => println!(""),
        _ => {
            for idx in 0..args.len() {
                if idx > 0 {
                    print!(" ");
                }
                match &args[idx] {
                    Block::Object(Token::Number(n)) => print!("{}", n),
                    Block::Object(Token::Text(t)) => {
                        // FIXME: this is a hack to implement unicode newline and tab characters
                        print!("{}", t.replace("\\n", "\u{000A}").replace("\\t", "\u{0009}"))
                    },
                    _ => print!("{}", args[idx]),
                };
            };
            print!("\n");
        },
    };
    Ok(Block::Null)
}


fn _type(_: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1);
    Ok(match args[0] {
        Block::Object(Token::Text(_)) => Block::Object(Token::Text("<Text>".to_string())),
        Block::Object(Token::Number(_)) => Block::Object(Token::Text("<Number>".to_string())),
        Block::Func(_) => Block::Object(Token::Text("<Func>".to_string())),
        Block::NativeFunc(_) => Block::Object(Token::Text("<NativeFunc>".to_string())),
        Block::List(_) => Block::Object(Token::Text("<List>".to_string())),
        Block::Bool(_) => Block::Object(Token::Text("<Bool>".to_string())),
        Block::Expr(_) => Block::Object(Token::Text("<Expr>".to_string())),
        Block::FuncBody(_) => Block::Object(Token::Text("<FuncBody>".to_string())),
        Block::Mod(_) => Block::Object(Token::Text("<Module>".to_string())),
        Block::Null => Block::Object(Token::Text("<Null>".to_string())),
        _ => Block::Object(Token::Text("<Type Not Found>".to_string()))
    })
}

// ================ if =======================

fn _if(_: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 3);
    let condition = match &args[0] {
        Block::List(l) => l[0].get_bool().unwrap(),
        Block::Bool(b) => *b,
        _ => eval_error!("unsupported condition statement")
    };

    let branch = if condition==true { 1 } else { 2 };
    Ok(args[branch].clone())
}


// ================ import ================

fn _import(ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1);
    match &args[0] {
        Block::Object(Token::Text(p)) => {
            let mut p = ns.get_relative_path(p);
            if !p.ends_with("krt") {
                p.set_extension("krt");
            }
            print_verbose!("import({:?})", p);
            let mut tokens = lexer::lex_file(&p)?;
            let tree = parser::parse(&mut tokens)?;

            let mut new_ns = NameSpace::new(Some(&p), Some(ns));
            new_ns.run(&tree)?;
            new_ns.to_block()
        },
        _ => Ok(Block::Null)
    }
}



fn _import_native(ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1);
    match &args[0] {
        Block::Object(Token::Text(p)) => {
            let mut p = ns.get_relative_path(&p);
            // let fname = libloading::library_filename(p.file_name().unwrap());
            helper::convert_dylib_os_name(&mut p);

            print_verbose!("import_native({:?})", p);

            let mut new_ns = NameSpace::new(Some(&p), Some(ns));
            new_ns.module.load_dylib();
            new_ns.to_block()
        },
        _ => Ok(Block::Null)
    }
}


fn _spill(ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1);
    match &args[0] {
        Block::Mod(m) => {
            ns.module.vars.extend(m.vars.clone());
        },
        _ => eval_error!("Unsupported argument to spill")
    };
    Ok(Block::Null)
}

// ================ iter ================

fn _len(_: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1);
    let length = match &args[0] {
        Block::List(l) => l.len(),
        Block::Object(Token::Text(t)) => t.len(),
        _ => eval_error!("len() not supported")
    };
    Ok(Block::Object(Token::Number(length as f64)))
}

fn _foreach(ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 2);
    if let Block::Func(_) | Block::NativeFunc(_) = &args[1] {
        let res: Vec<Block>;
        return match &args[0] {
            Block::List(l) => {
                res = NameSpace::resolve_vector(&l, &mut |x| ns.eval_func_obj(&args[1], &x, None))?;
                Ok(Block::List(res))
            },
            Block::Object(Token::Text(t)) => {
                let mut chars = t.chars().collect::<Vec<char>>();
                res = NameSpace::resolve_vector(&mut chars, &mut |c| ns.eval_func_obj(&args[1], &Block::Object(Token::Text(c.to_string())), None))?;
                // res = t.chars().map(|c| ns.eval_func_obj(&args[1], &Block::Object(Token::Text(c.to_string())), None)).collect();
                Ok(Block::List(res))
            },
            _ => eval_error!("iteration not supported")
        }
    } else {
        eval_error!("second argument should be a function");
    }
}

// ================ module inspect ================

fn _vars(ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_le!(args, 1); // 0 or 1 args
    let mut vars: Vec<Block> = Vec::new();
    if args.len() == 0 {
        for (k,_) in &ns.module.vars {
            vars.push(Block::Object(Token::Text(k.clone())));
        }
        Ok(Block::List(vars))
    }
    else if let Block::Mod(m) = &args[0] {
        for (k,_) in &m.vars {
            vars.push(Block::Object(Token::Text(k.clone())));
        }
        Ok(Block::List(vars))
    } else {
        Ok(Block::Null)
    }
}


// ================ process =======================

fn _exit(_ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 0);
    // std::process::exit(0);
    sys_exit_error!()
}


fn _assert(_ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1);
    let res = match &args[0] {
        Block::Bool(b) => *b==true,
        _ => eval_error!("assert argument not supported")
    };
    if res!=true {
        eval_error!("Assertion error");
    }
    Ok(args[0].clone())
}



// ================ namespace helper functions ====================


pub fn load_all(env_native: &mut HashMap<String, Block>) {
    env_native.insert("null".to_string(), Block::Null);
    env_native.insert("true".to_string(), Block::Bool(true));
    env_native.insert("false".to_string(), Block::Bool(false));

    helper::load_func(env_native, "print", _print);
    helper::load_func(env_native, "type", _type);
    helper::load_func(env_native, "if", _if);
    helper::load_func(env_native, "len", _len);
    helper::load_func(env_native, "foreach", _foreach);
    helper::load_func(env_native, "vars", _vars);

    helper::load_func(env_native, "import", _import);
    helper::load_func(env_native, "import_native", _import_native);
    helper::load_func(env_native, "spill", _spill);

    helper::load_func(env_native, "assert", _assert);
    helper::load_func(env_native, "exit", _exit);
}
