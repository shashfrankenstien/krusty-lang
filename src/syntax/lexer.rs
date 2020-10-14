use std::env; // required for print_verbose! macro
use regex::RegexSet;
use regex::Regex;
use lazy_static::lazy_static;


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
    Index(Box<Token>),
    Assign,
    Accessor,
    _Comment,
    _NewLine,
}


lazy_static! {
    static ref RE: RegexSet = RegexSet::new(&[
        r"^[\*]?[_a-zA-Z]+[_a-zA-Z0-9]*$", //symbol - 0
        r"^[+-]?[.\d]+$", //numbers - 1
        r#"(^".*"$)|(^'.*'$)"#, //strings1 - 2
        r#"^[+\-/\*]$"#, //Arith - 3
        r"^;$", //sep - 4
        r"^[({]$", //scopestart - 5
        r"^[})]$", //scopeend - 6
        r"^=>$", //funcDef - 7
        r"^,$", //List - 8
        r"^=$", //assign - 9
        r"^#.*$", //comment - 10
        r"^(\r\n|\r|\n)$", //newline - 11
        r#"^\[.*\]$"#, //index operation - 12
        r#"^(==|!=|<|<=|>|>=)$"#, //comparison operation - 13
    ]).unwrap();

    static ref RE_PASS: RegexSet = RegexSet::new(&[
        // continue parsing token if the following are encontered
        r#"^('|")[^'"]*$"#, //start of string
        r#"^(\[|\[['"])[^'"\]]*$"#, //start of index operation
    ]).unwrap();

    // Some parsing helpers
    static ref RE_QUOTES: Regex = Regex::new(r#"['"]"#).unwrap(); // for strings
    static ref RE_SQ_BRACKETS: Regex = Regex::new(r"[\[\]]").unwrap(); // for indexing operator
}




impl Token {

    fn _which_matched(txt: &str) -> Option<Token> {
        let m: Vec<_> = RE.matches(txt).into_iter().collect();
        if !m.is_empty() {
            return match m[0] {
                0 if txt == "ret" => Some(Token::FuncReturn),
                0 => Some(Token::Symbol(txt.to_string())),

                1 if txt == "." => Some(Token::Accessor), // hacky workaround due to lack of regex lookaround
                1 => Some(Token::Number(txt.parse().expect("This is not a number"))),

                2 => Some(Token::Text(RE_QUOTES.replace_all(txt, "").to_string())),
                3 => Some(Token::Arith(txt.chars().nth(0).unwrap())),
                5 => Some(Token::ScopeStart(txt.chars().nth(0).unwrap())),
                6 => Some(Token::ScopeEnd(txt.chars().nth(0).unwrap())),
                4 => Some(Token::Separator),
                7 => Some(Token::FuncDef),
                8 => Some(Token::List),
                9 => Some(Token::Assign),
                10 => Some(Token::_Comment),
                11 => Some(Token::_NewLine),
                12 => Some(Token::Index(Box::new(Token::create(&RE_SQ_BRACKETS.replace_all(txt, "")).unwrap()))),
                13 => Some(Token::Comparison(txt.to_string())),
                _ => None
            }
        } else {
            None
        }
    }

    pub fn is_newline_token(&self) -> bool {
        match self {
            Token::_NewLine => true,
            _ => false
        }
    }

    pub fn create(value: &str) -> Option<Token> {
        if RE_PASS.is_match(value) {
            return None
        }

        let token = match Token::_which_matched(value) {
            Some(k) => k,
            None => panic!("Illegal symbol {}", value)
        };
        Some(token)
    }
}




#[derive(Debug)]
pub struct Scanner {
    tokens: Vec<Token>,
    _pointer: usize,
}

impl Scanner {
    pub fn new(tokens: &Vec<Token>) -> Scanner {
        Scanner {
            tokens:tokens.to_vec(),
            _pointer:0,
        }
    }
    fn _valid_index(&self, i: usize) -> bool {
        i < self.tokens.len()
    }
    pub fn inc(&mut self) {
        self._pointer += 1;
    }
    // fn dec(&mut self) {
    //     self._pointer -= 1;
    // }

    pub fn current_idx(&self) -> usize {
        self._pointer
    }

    pub fn get_token_at(&self, i: usize) -> Option<&Token> {
        if self._valid_index(i) {
            Some(&self.tokens[i])
        } else {
            None
        }
    }

    pub fn get_token(&self) -> Option<&Token> {
        self.get_token_at(self._pointer)
    }

    pub fn get_next(&self) -> Option<&Token> {
        self.get_token_at(self._pointer + 1)
    }
    // fn get_prev(&self) -> Option<&Token> {
    //     self.get_token_at(self._pointer - 1)
    // }

    pub fn current_is(&self, other: &Option<Token>) -> bool {
        let tkn = self.get_token();
        match other {
            Some(t) => Some(t)==tkn,
            None => tkn.is_none()
        }
    }

    pub fn next_is(&self, other: &Option<Token>) -> bool {
        let tkn = self.get_next();
        match other {
            Some(t) => Some(t)==tkn,
            None => tkn.is_none()
        }
    }

}


fn push_tweaked(tkn: Token, dest: &mut Vec<Token>) {
    match &tkn {
        Token::ScopeStart(_) => {
            if let Token::Symbol(_) = dest[dest.len()-1] { // symbol + scope start = func call
                dest.push(Token::FuncCall);
            }
        },
        Token::Symbol(s) if s == "ret" => {
            dest.push(Token::FuncReturn);
            return
        },
        _ => ()
    };
    dest.push(tkn);
}



pub fn lex(code: String) -> Scanner {

    let mut word = String::new();
    let mut out: Vec<Token> = Vec::new();
    for c in code.chars() {
        word.push(c);
        if !RE.is_match(&word.trim_matches(' ')) && (word.trim_matches(' ')!="") {
            word.pop();
            // println!("{} {}", word, RE.is_match(&word));
            match Token::create(&word.trim_matches(' ')) {
                Some(t) => {
                    print_verbose!("{:?}", t);
                    push_tweaked(t, &mut out);
                    word.clear();
                },
                _ => ()
            };
            word.push(c);
        }
    }
    if word.len() != 0 { // check remainder
        match Token::create(&word.trim_matches(' ')) {
            Some(t) => push_tweaked(t, &mut out),
            _ => ()
        }
    }
    print_verbose!("\\mm/      lex done!!!");
    Scanner::new(&out)
}
