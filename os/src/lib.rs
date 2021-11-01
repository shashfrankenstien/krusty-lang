use std::path::PathBuf;
use std::ffi::OsStr;
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait
use std::fs;
use std::env;

use krusty_core::syntax::lexer::Token;
use krusty_core::syntax::parser::Block;
use krusty_core::syntax::evaluator::NameSpace;

#[macro_use] extern crate krusty_core;
use krusty_core::lib::{moddef, helper};
use krusty_core::lib::errors::{Error, KrustyErrorType};


fn _read_dir_to_list(dirpath: &PathBuf) -> Result<Vec<Block>, std::io::Error> {
    let dirpath = fs::canonicalize(dirpath).expect("No such File!");
    let mut v: Vec<Block> = Vec::new();
    for entry in fs::read_dir(dirpath)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap_or(OsStr::new("unknown")).to_str();
        v.push(Block::Object(Token::Text(name.unwrap_or("unknown").to_string())))
    }
    Ok(v)
}

pub fn _listdir(_ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_le!(args, 1); // 0 or 1 args
    match args.len() {
        0 => {
            let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
            Ok(Block::List(_read_dir_to_list(&cwd)?))
        }
        1 => {
            match &args[0] {
                Block::Object(Token::Text(t)) => {
                    let buf = PathBuf::from_slash(t);
                    Ok(Block::List(_read_dir_to_list(&buf)?))
                },
                _ => generic_error!("function only takes text")
            }
        },
        _ => Ok(Block::Null)
    }
}

#[no_mangle]
pub fn load_all(m_vars: &mut moddef::ModuleVars) {
    helper::load_func(m_vars, "listdir", _listdir);
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
