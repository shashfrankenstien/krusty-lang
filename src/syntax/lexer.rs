use regex::RegexSet;
use regex::Regex;
use lazy_static::lazy_static;


#[derive(Debug, Clone)]
pub enum Token {
    Symbol(String),
    Number(String),
    Text(String),
    Arith(String),
    Separator(String),
    ScopeStart(String),
    ScopeEnd(String),
    FuncDef(String),
    List(String),
    Assign(String),
    _Comment,
    _NewLine,
}


lazy_static! {
    static ref RE: RegexSet = RegexSet::new(&[
        r"^[a-zA-Z]+[a-zA-Z0-9]*$", //symbol
        r"^[.0-9]+$", //numbers
        r#"(^"\S*"$)|(^'\S*'$)"#, //strings1
        r#"^[+\-/\*]$"#, //Arith
        r"^;$", //sep
        r"^[({]$", //scopestart
        r"^[})]$", //scopeend
        r"^=>$", //funcDef
        r"^,$", //List
        r"^=$", //assign
        r"^#\S*$", //comment
        r"^(\r\n|\r|\n)$", //newline
    ]).unwrap();

    static ref RE_PASS: RegexSet = RegexSet::new(&[
        r#"^('|")$"#, //start of string
        r#"^('|")[^'"]*$"#, //start of string
    ]).unwrap();

    static ref RE_QUOTES: Regex = Regex::new(r#"['"]"#).unwrap();
}




impl Token {
    pub fn is_stmt_end_token(&self) -> bool {
        match self {
            Token::Separator(_) => true,
            // lexer::Token::ScopeEnd(_) => true,
            _ => false
        }
    }

    pub fn is_scope_end_token(&self) -> bool {
        match self {
            Token::ScopeEnd(_) => true,
            _ => false
        }
    }

    pub fn is_newline_token(&self) -> bool {
        match self {
            Token::_NewLine => true,
            _ => false
        }
    }

    fn tokenize(value: &str) -> Option<Token> {
        if RE_PASS.is_match(value) {
            return None
        }

        let token = match Token::_which_matched(value) {
            Some(k) => k,
            None => panic!("Illegal symbol {}", value)
        };
        Some(token)
    }

    fn _which_matched(txt: &str) -> Option<Token> {
        let m: Vec<_> = RE.matches(txt).into_iter().collect();
        if !m.is_empty() {
            return match m[0] {
                0 => Some(Token::Symbol(txt.to_string())),
                1 => Some(Token::Number(txt.to_string())),
                2 => Some(Token::Text(RE_QUOTES.replace_all(txt, "").to_string())),
                3 => Some(Token::Arith(txt.to_string())),
                4 => Some(Token::Separator(txt.to_string())),
                5 => Some(Token::ScopeStart(txt.to_string())),
                6 => Some(Token::ScopeEnd(txt.to_string())),
                7 => Some(Token::FuncDef(txt.to_string())),
                8 => Some(Token::List(txt.to_string())),
                9 => Some(Token::Assign(txt.to_string())),
                10 => Some(Token::_Comment),
                11 => Some(Token::_NewLine),
                _ => None
            }
        } else {
            None
        }
    }
}


pub fn lex(code: String) -> Vec<Token> {

    let mut word = String::new();
    let mut out = Vec::new();
    for c in code.chars() {
        word.push(c);
        if !RE.is_match(&word.trim_matches(' ')) && (word.trim_matches(' ')!="") {
            word.pop();
            // println!("{} {}", word, RE.is_match(&word));
            match Token::tokenize(&word.trim_matches(' ')) {
                Some(t) => {
                    // println!("{:?}", t);
                    out.push(t);
                    word.clear();
                },
                _ => ()
            };
            word.push(c);
        }
    }
    if word.len() != 0 { // check remainder
        match Token::tokenize(&word.trim_matches(' ')) {
            Some(t) => out.push(t),
            _ => ()
        }
    }
    out
}
