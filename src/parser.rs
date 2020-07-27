use crate::lexer;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Assign,
    Arith(char),
    List,
    Func,
    Call,
    Scope,
    Generic,
}

impl Operator {
    fn find(tok: &lexer::Token) -> Option<Operator> {
        match tok {
            lexer::Token::Arith(n) => Some(Operator::Arith(n.chars().nth(0).unwrap())),
            lexer::Token::FuncDef(_) => Some(Operator::Func),
            lexer::Token::Assign(_) => Some(Operator::Assign),
            lexer::Token::List(_) => Some(Operator::List),
            lexer::Token::ScopeStart(_) => Some(Operator::Scope),
            _ => None
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum Dtype {
    Num(f64),
    Text(String),
    Symbol(String),
    Expr(Expression),
    Grouped(ExprList),
}

impl Dtype {
    fn find(tok: &lexer::Token) -> Option<Dtype> {
        match tok {
            lexer::Token::Symbol(n) => Some(Dtype::Symbol(n.clone())),
            lexer::Token::Number(n) => Some(Dtype::Num(n.parse().expect("This is not a number"))),
            lexer::Token::Text(n) => Some(Dtype::Text(n.clone())),
            _ => None
        }
    }
}


#[derive(Debug, PartialEq)]
pub struct Expression {
    op: Operator,
    elems: Vec<Dtype>,
    _op_found: bool,
}


impl Expression {

    fn _stmt_end_token(t: &lexer::Token) -> bool {
        match t {
            lexer::Token::Separator(_) => true,
            // lexer::Token::ScopeEnd(_) => true,
            _ => false
        }
    }

    fn _scope_end_token(t: &lexer::Token) -> bool {
        match t {
            lexer::Token::ScopeEnd(_) => true,
            _ => false
        }
    }

    fn new() -> Expression {
        Expression {
            op: Operator::Generic,
            elems: Vec::new(),
            _op_found: false,
        }
    }


    fn parse(&mut self, tokens: &mut Scanner) {
        println!("-->");
        // println!("{:?}", tokens);
        // let mut pos: usize = 0;
        // let mut scope_start = 0;
        // if tokens.len()==0 {
        //     return 0;
        // }
        loop {
            let tok = tokens.get_token();
            if tok.is_none() {
                break;
            }
            let tok = tok.unwrap();
            println!("{:?}", tok);
            if Expression::_stmt_end_token(&tok) || Expression::_scope_end_token(&tok) {
                println!("*****");
                println!("{:?}", self);
                println!("break");
                println!("*****");
                tokens.inc();
                break;
            }
            if let Some(d) = Dtype::find(&tok) {
                if !self._op_found {
                    self.elems.push(d);
                } else {
                    println!("RHS -++");
                    let mut exp = Expression::new();
                    tokens.inc();// look forward
                    exp.parse(tokens);
                    // pos += _pos;
                    if exp._op_found {
                        exp.elems.push(d);
                        exp.elems.rotate_right(1); //moving d to first elem
                        self.elems.push(Dtype::Expr(exp));
                    } else if exp.elems.len() == 0 {
                        self.elems.push(d);
                    } else {
                        panic!("What happened here! {:?} {:?}", d, exp);
                    }
                    break;
                    println!("RHS ---");
                }
            } else if let Some(o) = Operator::find(&tok) {
                if o==Operator::Scope {
                    if !self._op_found && self.elems.len()==1 {
                        if let Dtype::Symbol(_)=self.elems[0] {
                            println!("FUNC CALL");
                            self._op_found = true;
                            self.op = Operator::Call;
                        }
                    }
                    let mut expl = ExprList::new();
                    tokens.inc(); //skip the scope start symbol
                    expl.parse(tokens);
                    // pos += _pos;
                    if let Some(nxt_t) = tokens.get_token() {
                        if *nxt_t==lexer::Token::FuncDef("=>".to_string()) { //Handle ()=>{} function definition
                            let mut exp = Expression::new();
                            exp.parse(tokens);
                            exp.elems.push(Dtype::Grouped(expl));
                            exp.elems.rotate_right(1); //moving d to first elem
                            self.elems.push(Dtype::Expr(exp));
                            break;
                        } else {
                            self.elems.push(Dtype::Grouped(expl));
                        }
                    }
                    // println!("- Scope end {:?}", &tokens[pos..]);
                    println!("- Scope end SELF {:?}", self);
                    continue;
                } else if !self._op_found {
                    println!("{:?}", o);
                    self._op_found = true;
                    self.op = o;
                } else {
                    if o==Operator::Func {
                        println!("- FUNCTIOOOOOONNN {:?}", self);
                    }
                    let mut exp = Expression::new();
                    exp.parse(tokens);
                    if exp.elems.len()!=0 {
                        self.elems.push(Dtype::Expr(exp));
                    }
                }
            }
            // pos+=1;
            tokens.inc();
        }
        println!("<--");
        // pos
    }
}

#[derive(Debug, PartialEq)]
pub struct ExprList {
    exprs: Vec<Expression>
}

impl ExprList {
    fn new() -> ExprList {
        ExprList {
            exprs: Vec::new(),
        }
    }

    fn parse(&mut self, tokens: &mut Scanner) {
        println!("-->>>>>>>");
        // println!("{:?}", tokens);
        // let mut pos: usize = 0;
        loop {
            // let tok = tokens.get_token();
            let mut exp = Expression::new();
            exp.parse(tokens);
            println!("=========");
            // println!("{} {} {:?}", pos, pos+npos, exp);
            println!("{} {:?}", tokens._pointer, exp);
            println!("{}", exp.elems.len());
            // println!("{:?}", &tokens[pos+npos-1]);
            println!("=========");
            if exp.elems.len()>0 {
                self.exprs.push(exp);
            }

            let mut scope_ended = false;
            if let Some(t)=tokens.get_prev() {
                scope_ended = Expression::_scope_end_token(t); // very hacky
            }
            // pos += npos;
            if tokens.get_token().is_none() || scope_ended==true {
                break;
            }
        }
        println!("<<<<<<<<---");
        // pos
    }
}


#[derive(Debug)]
struct Scanner {
    tokens: Vec<lexer::Token>,
    _pointer: usize,
}

impl Scanner {
    fn new(tokens: &Vec<lexer::Token>) -> Scanner {
        Scanner {
            tokens:tokens.to_vec(),
            _pointer:0,
        }
    }
    fn _valid_index(&self, i: usize) -> bool {
        i < self.tokens.len()
    }
    fn inc(&mut self) {
        self._pointer += 1;
        println!("{} {}", self._pointer, self.tokens.len());
    }
    fn dec(&mut self) {
        self._pointer -= 1;
    }
    fn get_token(&self) -> Option<&lexer::Token> {
        if self._valid_index(self._pointer) {
            Some(&self.tokens[self._pointer])
        } else {
            None
        }
    }
    fn get_prev(&self) -> Option<&lexer::Token> {
        if self._valid_index(self._pointer-1) {
            Some(&self.tokens[self._pointer-1])
        } else {
            None
        }
    }
    fn get_next(&self) -> Option<&lexer::Token> {
        if self._valid_index(self._pointer+1) {
            Some(&self.tokens[self._pointer+1])
        } else {
            None
        }
    }

}


pub fn parse(tokens: Vec<lexer::Token>) {
    // println!("{:?}", tokens);
    // let e = Expression::new(&tokens);
    // for ex in &e {
    //     println!("{:?}", ex);
    // }
    let mut scan = Scanner::new(&tokens);
    let mut output = ExprList::new();
    output.parse(&mut scan);
    for o in &output.exprs {
        println!("{:?}", o)
    }
    // // let mut exp = Expression::new();
    // // let mut prev_elem: Option<Dtype> = None;
    // let mut pos = 0;
    // loop {
    //     let tok = &tokens[pos..];
    //     let mut exp = Expression::new();
    //     let npos = exp.parse(tok);

    //     println!("=========");
    //     println!("{} {:?}", pos, exp);
    //     println!("{:?}", &tokens[pos..pos+npos]);
    //     pos += npos;
    //     output.push(exp);
    //     if tok.len()==0 {
    //         break;
    //     }
    // }
}
