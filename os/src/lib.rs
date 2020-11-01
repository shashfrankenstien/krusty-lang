use std::path::PathBuf;
use std::ffi::OsStr;
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait
use std::fs;
use std::env;
use std::collections::HashMap;

use krusty_core::syntax::lexer::Token;
use krusty_core::syntax::parser::Obj;
use krusty_core::syntax::evaluator::NameSpace;

use krusty_core::lib::loader;


fn _read_dir_to_list(dirpath: &PathBuf) -> Result<Vec<Obj>, std::io::Error> {
    let dirpath = fs::canonicalize(dirpath).expect("No such File!");
    let mut v: Vec<Obj> = Vec::new();
    for entry in fs::read_dir(dirpath)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap_or(OsStr::new("unknown")).to_str();
        v.push(Obj::Object(Token::Text(name.unwrap_or("unknown").to_string())))
    }
    Ok(v)
}

pub fn _listdir(_: &mut NameSpace, args: &Vec<Obj>) -> Obj {
    println!("{:?}", args);
    if args.len() > 1 {
        panic!("takes 0 or 1 arguments, {} supplied", args.len())
    }
    match args.len() {
        0 => {
            let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
            match _read_dir_to_list(&cwd) {
                Ok(v) => Obj::List(v),
                Err(e) => panic!(e)
            }
        }
        1 => {
            match &args[0] {
                Obj::Object(Token::Text(t)) => {
                    let buf = PathBuf::from_slash(t);
                    match _read_dir_to_list(&buf) {
                        Ok(v) => Obj::List(v),
                        Err(e) => panic!(e)
                    }
                },
                _ => panic!("function only takes text")
            }
        },
        _ => Obj::Null
    }
}

#[no_mangle]
pub fn load(ns_vars: &mut HashMap<String, Obj>) {
    loader::load_func(ns_vars, "listdir", _listdir);
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
