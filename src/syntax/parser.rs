use crate::syntax::lexer;

#[derive(Debug, Clone)]
pub enum Operator {
    Assign,
    Arith(char),
    List,
    Func,
    Scope,
    Call,
    NotFound,
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


#[derive(Debug, Clone)]
pub enum Obj {
    Num(f64),
    Text(String),
    Symbol(String),
    Expr(Expression),
    Grouped(ExprList),
}

impl Obj {
    fn find(tok: &lexer::Token) -> Option<Obj> {
        match tok {
            lexer::Token::Symbol(n) => Some(Obj::Symbol(n.clone())),
            lexer::Token::Number(n) => Some(Obj::Num(n.parse().expect("This is not a number"))),
            lexer::Token::Text(n) => Some(Obj::Text(n.clone())),
            _ => None
        }
    }

}

// impl Copy for Obj {

// }


#[derive(Debug, Clone)]
pub struct Expression {
    pub op: Operator,
    pub elems: Vec<Obj>,
}


impl Expression {

    fn _op_found(&self) -> bool {
        match self.op {
            Operator::NotFound => false,
            _ => true
        }
    }

    fn new() -> Expression {
        Expression {
            op: Operator::NotFound,
            elems: Vec::new(),
        }
    }


    fn parse(&mut self, tokens: &mut Scanner) {
        // println!("-->");
        // println!("{:?}", tokens);
        loop {
            let tok = tokens.get_token();
            if tok.is_none() {
                break;
            }
            let tok = tok.unwrap();
            // println!("{:?}", tok);
            if tok.is_stmt_end_token() || tok.is_scope_end_token() {
                tokens.inc();
                break;
            } else if let lexer::Token::_Comment = tok {
                // loop till EOL
                loop {
                    tokens.inc();
                    if let Some(t) = tokens.get_token() {
                        if t.is_newline_token() {
                            break;
                        }
                    }
                }
                tokens.inc();
                break;
            } else if let Some(d) = Obj::find(&tok) {
                if !self._op_found() {
                    self.elems.push(d);
                } else {
                    // println!("RHS -++");
                    let mut exp = Expression::new();
                    tokens.inc();// look forward
                    exp.parse(tokens);
                    if exp._op_found() {
                        exp.elems.push(d);
                        exp.elems.rotate_right(1); //moving d to first elem
                        self.elems.push(Obj::Expr(exp));
                    } else if exp.elems.len() == 0 {
                        self.elems.push(d);
                    } else {
                        panic!("What happened here! {:?} {:?}", d, exp);
                    }
                    // println!("RHS ---");
                    break;
                }
            } else if let Some(o) = Operator::find(&tok) {
                if let Operator::Scope = o {
                    if !self._op_found() && self.elems.len()==1 {
                        if let Obj::Symbol(_)=self.elems[0] {
                            // println!("FUNC CALL");
                            self.op = Operator::Call;
                        }
                    }
                    let mut expl = ExprList::new();
                    tokens.inc(); //skip the scope start symbol
                    expl.parse(tokens);
                    if let Some(nxt_t) = tokens.get_token() {
                        if let lexer::Token::FuncDef(_) = *nxt_t { //Handle ()=>{} function definition
                            let mut exp = Expression::new();
                            exp.elems.push(Obj::Grouped(expl));
                            exp.parse(tokens);
                            self.elems.push(Obj::Expr(exp));
                            break;
                        } else {
                            self.elems.push(Obj::Grouped(expl));
                        }
                    }
                    // println!("- Scope end SELF {:?}", self);
                    continue; //skip final increment
                } else if !self._op_found() {
                    // println!("{:?}", o);
                    self.op = o;
                } else {
                    let mut exp = Expression::new();
                    exp.parse(tokens);
                    if exp.elems.len()!=0 {
                        self.elems.push(Obj::Expr(exp));
                    }
                }
            }
            tokens.inc();
        }
        // println!("<--");
    }
}

#[derive(Debug, Clone)]
pub struct ExprList {
    pub exprs: Vec<Expression>,
}

impl ExprList {
    fn new() -> ExprList {
        ExprList {
            exprs: Vec::new(),
        }
    }

    fn parse(&mut self, tokens: &mut Scanner) {
        // println!("-->>>>>>>");
        loop {
            let mut exp = Expression::new();
            exp.parse(tokens);
            // println!("=========");
            // println!("{} {:?}", tokens._pointer, exp);
            // println!("{}", exp.elems.len());
            // println!("=========");
            if exp.elems.len()>0 {
                self.exprs.push(exp);
            }

            let mut scope_ended = false;
            if let Some(t)=tokens.get_prev() {
                scope_ended = t.is_scope_end_token();
            }
            if tokens.get_token().is_none() || scope_ended==true {
                break;
            }
        }
        // println!("<<<<<<<<---");
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
        // println!("{} {}", self._pointer, self.tokens.len());
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


pub fn parse(tokens: Vec<lexer::Token>) -> ExprList {

    let mut scan = Scanner::new(&tokens);
    let mut output = ExprList::new();
    output.parse(&mut scan);
    for (i,o) in output.exprs.iter().enumerate() {
        println!("{} {:?}", i, o)
    }
    println!("------------------");
    output
}
