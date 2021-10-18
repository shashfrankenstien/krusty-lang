
use super::lexer::Token;



pub fn push_tweaked(tkn: Token, dest: &mut Vec<Token>) {
    match &tkn {
        Token::ScopeStart('(') => {
            // if let Token::Symbol(_)  = dest[dest.len()-1] { // symbol + scope start = func call
            if dest.len() > 0 {
                if let Token::Symbol(_) | Token::ScopeEnd(_) = dest[dest.len()-1] { // symbol + scope start = func call
                    dest.push(Token::FuncCall);
                }
            }
            dest.push(tkn);
        },
        Token::ScopeStart('[') => {
            dest.push(Token::Index);
            dest.push(tkn);
        },
        Token::Symbol(s) if s == "ret" => {
            dest.push(Token::FuncReturn);
            return
        },
        _ => dest.push(tkn)
    };
}
