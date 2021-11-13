use std::path::PathBuf;
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait
use std::fs::{File, OpenOptions};
use std::io::{Read, BufReader, BufRead, BufWriter, Write};

use krusty_core::syntax::evaluator::NameSpace;
use krusty_core::syntax::lexer::Token;
use krusty_core::syntax::parser::Block;

use krusty_core::lib::moddef::Module;
use krusty_core::lib::errors::{Error, KrustyErrorType};
use krusty_core::lib::helper;




fn create_filemodule(filepath: &String) -> Module {
	let mut fobj = Module::new(None);
	fobj.vars.insert("filepath".to_string(), Block::Object(Token::Text(filepath.clone())));
	helper::load_func(&mut fobj.vars, "read", _read);
	helper::load_func(&mut fobj.vars, "read_all", _read_all);
	helper::load_func(&mut fobj.vars, "write", _write);
	helper::load_func(&mut fobj.vars, "append", _append);
	fobj
}

pub fn _fileopen(_ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1); // 1 args
	match &args[0] {
		Block::Object(Token::Text(f)) => {
			let filepath = PathBuf::from_slash(f);
			if !filepath.is_file() {
				eval_error!("File not found");
			}
			let filepath_str = filepath.to_str().ok_or("what")?.to_string();
			Ok(Block::Mod(create_filemodule(&filepath_str)))
		},
		_ => eval_error!("Unsupported argument")
	}
}

pub fn _filecreate(_ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1); // 1 args
	match &args[0] {
		Block::Object(Token::Text(f)) => {
			let filepath = PathBuf::from_slash(f);
			if filepath.is_file() {
				eval_error!("File exists");
			}
			let filepath_str = filepath.to_str().ok_or("what")?.to_string();
			File::create(&filepath_str)?;
			Ok(Block::Mod(create_filemodule(&filepath_str)))
		},
		_ => eval_error!("Unsupported argument")
	}
}

fn _read_all(ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 0); // 0 args
	let fpath = ns.get(&"filepath".to_string())?;
	match fpath {
		Block::Object(Token::Text(f)) => {
			let mut file = OpenOptions::new().read(true).open(f)?;
			let mut contents = String::new();
			file.read_to_string(&mut contents)?;
			Ok(Block::Object(Token::Text(contents)))
		},
		_ => eval_error!("File read error")
	}
}

fn _read(ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1); // 0 args
	let fpath = ns.get(&"filepath".to_string())?;
	match (fpath, &args[0]) {
		(Block::Object(Token::Text(f)), Block::Object(Token::Number(n))) => {
			let file = OpenOptions::new().read(true).open(f)?;
			let mut buf = BufReader::with_capacity(*n as usize, file);
			let contents = String::from_utf8_lossy(buf.fill_buf()?).into_owned();
			Ok(Block::Object(Token::Text(contents)))
		},
		_ => eval_error!("File read error")
	}
}


fn _write(ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1); // 0 args
	let fpath = ns.get(&"filepath".to_string())?;
	match (fpath, &args[0]) {
		(Block::Object(Token::Text(f)), Block::Object(Token::Text(t))) => {
			let file = OpenOptions::new().write(true).open(f)?;
			let mut buffer = BufWriter::new(file);
			buffer.write_all(t.as_bytes())?;
			buffer.flush()?;
			Ok(Block::Null)
		},
		_ => eval_error!("File write error")
	}
}


fn _append(ns: &mut NameSpace, args: &Vec<Block>) -> Result<Block, KrustyErrorType> {
    func_nargs_eq!(args, 1); // 0 args
	let fpath = ns.get(&"filepath".to_string())?;
	match (fpath, &args[0]) {
		(Block::Object(Token::Text(f)), Block::Object(Token::Text(t))) => {
			let file = OpenOptions::new().append(true).open(f)?;
			let mut buffer = BufWriter::new(file);
			buffer.write_all(t.as_bytes())?;
			buffer.flush()?;
			Ok(Block::Null)
		},
		_ => eval_error!("File read error")
	}
}
