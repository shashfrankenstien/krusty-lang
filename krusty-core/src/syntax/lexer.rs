#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro

use regex::RegexSet;
use lazy_static::lazy_static;

use std::io::Read; // for read_to_string
use std::fs;
use std::path::PathBuf;
use std::fmt;

use crate::lib::errors::{Error, KrustyErrorType};
use super::lexer_tweaks;


lazy_static! {
    static ref RE: RegexSet = RegexSet::new(&[
        r"^[\*]?[_a-zA-Z]+[_a-zA-Z0-9]*$", //symbol - 0
        r"^[+-]?[.\d]+$", //numbers - 1
        r#"(?s)(^".*"$)|(^'.*'$)"#, //strings - 2
        r#"^[+\-/\*]$"#, //Arith - 3
        r"^;$", //sep - 4
        r"^[({\[]$", //scopestart - 5
        r"^[})\]]$", //scopeend - 6
        r"^=>$", //funcDef - 7
        r"^,$", //List - 8
        r"^=$", //assign - 9
        r"^#.*$", //comment - 10
        r"^(\r\n|\r|\n)$", //newline - 11
        r#"^(==|!=|<|<=|>|>=)$"#, //comparison operation - 12
    ]).unwrap();

    static ref RE_PASS: RegexSet = RegexSet::new(&[
        // continue parsing token if the following are encountered
        r#"^('[^']*|"[^"]*)$"#, //start of string
    ]).unwrap();
}



#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    Number(f64),
    Text(String),
    Symbol(String),
    Arith(char),
    Comparison(String),
    ScopeStart(char),
    ScopeEnd(char),
    Separator,
    FuncDef,
    FuncCall,
    FuncReturn,
    List,
    Index,
    Assign,
    Accessor,
    _Comment,
    _NewLine,
}

// impl Drop for Token {
//     fn drop(&mut self) { println!("Dropping Token {:?}", self); }
// }

// impl Drop for TokenStream {
//     fn drop(&mut self) { println!("Dropping TokenStream {:?}", self); }
// }

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Text(t) => write!(f, "\"{}\"", t),
            _ => write!(f, "{:?}", self),
        }
    }
}



impl Token {

    fn _which_matched(txt: &str) -> Result<Token, KrustyErrorType> {
        // println!("{}", txt);
        let m: Vec<_> = RE.matches(txt).into_iter().collect();
        if !m.is_empty() {
            return match m[0] {
                0 if txt == "ret" => Ok(Token::FuncReturn),
                0 => Ok(Token::Symbol(txt.to_string())),

                1 if txt == "." => Ok(Token::Accessor), // hacky workaround due to lack of regex lookaround
                1 => Ok(Token::Number(txt.parse().expect("This is not a number"))),

                2 => Ok(Token::Text(txt[1..txt.len()-1].to_string())), // excluding quotes
                3 => Ok(Token::Arith(txt.chars().nth(0).unwrap())),
                5 => Ok(Token::ScopeStart(txt.chars().nth(0).unwrap())),
                6 => Ok(Token::ScopeEnd(txt.chars().nth(0).unwrap())),
                4 => Ok(Token::Separator),
                7 => Ok(Token::FuncDef),
                8 => Ok(Token::List),
                9 => Ok(Token::Assign),
                10 => Ok(Token::_Comment),
                11 => Ok(Token::_NewLine),
                12 => Ok(Token::Comparison(txt.to_string())),
                _ => lex_error!("Unidentified symbol")
            }
        } else {
            lex_error!("Unidentified symbol")
        }
    }

    pub fn is_newline_token(&self) -> bool {
        match self {
            Token::_NewLine => true,
            _ => false
        }
    }

    pub fn create(value: &str) -> Result<Token, KrustyErrorType> {
        Token::_which_matched(value)
    }
}




#[derive(Debug)]
pub struct TokenStream {
    tokens: Vec<Token>,
    _pointer: usize,
}

impl TokenStream {
    pub fn new() -> TokenStream {
        TokenStream {
            tokens: Vec::new(),
            _pointer:0,
        }
    }

    fn _valid_index(&self, i: usize) -> bool {
        i < self.tokens.len()
    }

    pub fn inc_n(&mut self, n: usize) {
        self._pointer += n;
    }
    pub fn inc(&mut self) {
        self.inc_n(1);
    }
    pub fn dec_n(&mut self, n: usize) {
        self._pointer -= n;
    }
    pub fn dec(&mut self) {
        self.dec_n(1);
    }

    pub fn current_idx(&self) -> usize {
        self._pointer
    }

    pub fn get_current_at(&self, i: usize) -> Option<&Token> {
        if self._valid_index(i) {
            Some(&self.tokens[i])
        } else {
            None
        }
    }

    pub fn get_current(&self) -> Option<&Token> {
        self.get_current_at(self._pointer)
    }

    pub fn get_next(&self) -> Option<&Token> {
        self.get_current_at(self._pointer + 1)
    }

    pub fn get_prev(&self) -> Option<&Token> {
        self.get_current_at(self._pointer - 1)
    }

    fn is(tkn: Option<&Token>, other: &Option<Token>) -> bool {
        match other {
            Some(t) => Some(t)==tkn,
            None => tkn.is_none()
        }
    }

    fn is_in(tkn: Option<&Token>, others: &Option<&[Token]>) -> bool {
        others.unwrap_or(&[]).iter().any(|x| Some(x)==tkn)
    }

    pub fn current_is(&self, other: &Option<Token>) -> bool {
        let tkn = self.get_current();
        TokenStream::is(tkn, other)
    }

    pub fn next_is(&self, other: &Option<Token>) -> bool {
        let tkn = self.get_next();
        TokenStream::is(tkn, other)
    }

    pub fn prev_is(&self, other: &Option<Token>) -> bool {
        let tkn = self.get_prev();
        TokenStream::is(tkn, other)
    }

    pub fn current_is_in(&self, others: &Option<&[Token]>) -> bool {
        let tkn = self.get_current();
        TokenStream::is_in(tkn, others)
    }

    pub fn next_is_in(&self, others: &Option<&[Token]>) -> bool {
        let tkn = self.get_next();
        TokenStream::is_in(tkn, others)
    }

    pub fn prev_is_in(&self, others: &Option<&[Token]>) -> bool {
        let tkn = self.get_prev();
        TokenStream::is_in(tkn, others)
    }
}


fn trim_spaces(w: &String) -> &str {
    w.trim_matches(&[' ', '\t'] as &[_])
}

pub fn lex(code: &String) -> Result<TokenStream, KrustyErrorType> {

    let mut word = String::new();
    let mut out: TokenStream = TokenStream::new();
    for c in code.chars() {
        word.push(c);
        if trim_spaces(&word).len()>1 && !RE.is_match(trim_spaces(&word)) { // if it matches, continue till it doesn't match
            word.pop();
            // println!("{} {}", word, RE.is_match(&word));

            if !RE_PASS.is_match(&word) {
                let t = Token::create(trim_spaces(&word))?;
                print_verbose!("{:?}", t);
                lexer_tweaks::push_tweaked(t, &mut out.tokens);
                word.clear();
            }
            word.push(c);
        }
    }
    if word.len() != 0 && !RE_PASS.is_match(&word) { // check remainder
        let t = Token::create(trim_spaces(&word))?;
        lexer_tweaks::push_tweaked(t, &mut out.tokens)
    }
    print_verbose!("\\mm/      lex done!!!");
    return Ok(out);
}


pub fn lex_file(filepath: &PathBuf) -> Result<TokenStream, KrustyErrorType> {
    let filepath = fs::canonicalize(filepath).expect("No such File!");
    let mut f = fs::File::open(filepath).expect("Oh, no such file!");
    let mut code = String::new();
    f.read_to_string(&mut code)?;//.expect("Can't read this");

    lex(&code)
}
