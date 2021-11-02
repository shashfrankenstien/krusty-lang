use std::convert::From;
use std::any::Any;
use std::io;
use std::fmt;


pub trait KrustyError {

	fn name(&self) -> String;
	fn msg(&self) -> &String;
	fn print_traceback(&self);

	fn as_any(&self) -> &dyn Any;
}

pub type KrustyErrorType = Box<dyn KrustyError>;



#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Error {
	GenericError{msg: String, fname: String, lino: i32},
	LexerError{msg: String, fname: String, lino: i32},
	ParserError{msg: String, fname: String, lino: i32},
	EvalError{msg: String, fname: String, lino: i32},
	SysExit{msg: String, fname: String, lino: i32},
}

impl KrustyError for Error {

	fn name(&self) -> String {
		match self {
			Error::GenericError{..} => "GenericError".to_string(),
			Error::LexerError{..} => "LexerError".to_string(),
			Error::ParserError{..} => "ParserError".to_string(),
			Error::EvalError{..} => "EvalError".to_string(),
			Error::SysExit{..} => "SysExit".to_string(),
		}
	}

	fn msg(&self) -> &String {
		match self {
			Error::GenericError{msg, ..} => msg,
			Error::LexerError{msg, ..} => msg,
			Error::ParserError{msg, ..} => msg,
			Error::EvalError{msg, ..} => msg,
			Error::SysExit{msg, ..} => msg,
		}
	}

	fn print_traceback(&self) {
		println!("**KrustyError**");
		println!("{}: {}", self.name(), self.msg());
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}


impl fmt::Display for Box<dyn KrustyError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg())
    }
}


impl From<io::Error> for Box<dyn KrustyError> {
	fn from(e: io::Error) -> Box<dyn KrustyError> {
		Box::new(Error::LexerError{msg: e.to_string(), fname: String::from(""), lino: -1})  // io:Error happens in Lexer while reading a file
	}
}



impl From<&'static str> for Box<dyn KrustyError> {
	fn from(e: &'static str) -> Box<dyn KrustyError> {
		Box::new(Error::GenericError{msg: e.to_string(), fname: String::from(""), lino: -1})  // convert string to error
	}
}

