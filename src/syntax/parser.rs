use crate::syntax::lexer;



#[derive(Debug, Clone)]
pub enum Obj {
    Object(lexer::Token),
    Operator(lexer::Token),
    Scope(lexer::Token),
    Expr(Box<Expression>),
    Group(ExprList),
    Null
}

impl Obj {
    fn find(tok: &lexer::Token) -> Obj {
        use lexer::Token;
        match tok {
            Token::Symbol(_) | Token::Number(_) | Token::Text(_) => Obj::Object(tok.clone()),
            Token::Arith(_) | Token::FuncDef | Token::Assign | Token::List | Token::FuncCall => Obj::Operator(tok.clone()),
            Token::ScopeStart(_) => Obj::Scope(tok.clone()),
            _ => Obj::Null
        }
    }
}




#[derive(Debug, Clone)]
pub struct Expression {
    pub op: Obj,
    pub elems: Vec<Obj>,
}


impl Expression {

    fn new() -> Expression {
        Expression {
            op: Obj::Null,
            elems: Vec::new(),
        }
    }

    fn to_object(self) -> Obj {
        if self.elems.len()==1 {
            self.elems[0].clone()
        } else {
            Obj::Expr(Box::new(self))
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
                loop { // loop till EOL
                    tokens.inc();
                    if let Some(t) = tokens.get_token() {
                        if t.is_newline_token() {
                            break;
                        }
                    }
                };
                tokens.inc();
                break;
            }
            let obj = Obj::find(&tok);
            match obj {
                Obj::Object(_) => {
                    if let Obj::Null = self.op {
                        self.elems.push(obj);
                    } else if let Obj::Operator(lexer::Token::List) = self.op {
                        self.elems.push(obj);
                    } else {
                        // println!("RHS -++");
                        let mut exp = Expression::new();
                        tokens.inc();// look forward
                        exp.parse(tokens);
                        if exp.elems.len() != 0 {
                            exp.elems.push(obj);
                            exp.elems.rotate_right(1); //moving obj to first elem
                            self.elems.push(exp.to_object());
                        } else {
                            self.elems.push(obj);
                        }
                        // println!("RHS ---");
                        break;
                    }
                },
                Obj::Scope(_) => {
                    let mut exp_list = ExprList::new();
                    tokens.inc(); //skip the scope start symbol
                    exp_list.parse(tokens);
                    let exp_obj = exp_list.to_object();
                    if let Some(nxt_t) = tokens.get_token() {
                        if let lexer::Token::FuncDef = *nxt_t { //Handle ()=>{} function definition
                            let mut exp = Expression::new();
                            exp.elems.push(exp_obj);
                            exp.parse(tokens);
                            self.elems.push(exp.to_object());
                            break;
                        } else {
                            self.elems.push(exp_obj);
                        }
                    }
                    continue; //skip final increment
                },
                Obj::Operator(op) => {
                    if let Obj::Null = self.op {
                        self.op = Obj::Operator(op);
                    } else if let Obj::Operator(lexer::Token::List) = self.op {
                        self.op = Obj::Operator(op);
                    } else {
                        let mut exp = Expression::new();
                        if self.elems.len() > 0 {
                            exp.elems.push(self.elems.pop().unwrap());
                        }
                        exp.parse(tokens);
                        if exp.elems.len()!=0 {
                            self.elems.push(exp.to_object());
                        }
                        break;
                    }
                },
                _=>()
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
            if exp.elems.len()>0 {
                if let Obj::Null = exp.op {
                    exp.op = Obj::Operator(lexer::Token::List)
                }
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

    fn to_object(mut self) -> Obj {
        if self.exprs.len()==0 {
            Obj::Null
        } else if self.exprs.len()==1 {
            self.exprs.pop().unwrap().to_object()
        } else {
            Obj::Group(self)
        }
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
    }
    // fn dec(&mut self) {
    //     self._pointer -= 1;
    // }
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
    // fn get_next(&self) -> Option<&lexer::Token> {
    //     if self._valid_index(self._pointer+1) {
    //         Some(&self.tokens[self._pointer+1])
    //     } else {
    //         None
    //     }
    // }

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
