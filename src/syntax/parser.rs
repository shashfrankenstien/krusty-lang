use std::env; // required for print_verbose! macro
use crate::syntax::lexer;


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FuncDef {
    pub args: Obj,
    pub body: Obj
}



#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Obj {
    Null,
    Bool(bool),
    Object(lexer::Token),
    Operator(lexer::Token),
    SuffixOperator(lexer::Token),
    Scope(char),
    Expr(Box<Expression>), // use Box since Expression has Obj type members (recursive)
    Group(Vec<Expression>),
    // Group(Vec<Obj>),
    List(Vec<Obj>),
    Func(Box<FuncDef>),
    BuiltinFunc(String),
}

impl Obj {
    fn categorize(tok: &lexer::Token) -> Obj {
        use lexer::Token;
        match tok {
            Token::Symbol(_)
                | Token::Number(_)
                | Token::Text(_)
                => Obj::Object(tok.clone()),

            Token::Arith(_)
                | Token::Comparison(_)
                | Token::FuncDef
                | Token::Assign
                | Token::List
                | Token::FuncCall
                | Token::FuncReturn
                => Obj::Operator(tok.clone()),

            Token::Index(_) => Obj::SuffixOperator(tok.clone()),
            Token::ScopeStart(s) => Obj::Scope(*s),
            _ => Obj::Null
        }
    }

    pub fn get_list(&self) -> Option<Vec<Obj>> {
        match self {
            Obj::List(l) => Some(l.clone()),
            _ => None,
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        match self {
            Obj::Bool(b) => Some(*b),
            _ => None,
        }
    }
}




#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

    pub fn to_object(mut self) -> Obj {
        if self.op == Obj::Null && self.elems.len() == 1 {
            self.elems.pop().unwrap()
        } else
        if let Obj::Operator(lexer::Token::List) | Obj::Null = self.op {
            Obj::List(self.elems)
        } else
        if let Obj::Operator(lexer::Token::FuncDef) = self.op {
            if self.elems.len() == 2 {
                let body = self.elems.pop().unwrap();
                let mut args = self.elems.pop().unwrap();
                // Obj::Func(FuncDef {args, body})
                match args { // convert args to list
                    Obj::List(_) => (),
                    _ => args = Obj::List(vec![args])
                }
                match body {
                    Obj::Group(_) | Obj::Expr(_) => (),
                    _ => panic!("Invalid function body")
                }
                Obj::Func(Box::new(FuncDef {args, body}))
            } else {
                panic!("Illegal function definition - {:?}", self)
            }
        }
        else {
            Obj::Expr(Box::new(self))
        }
    }


    fn parse_scope(tokens: &mut lexer::Scanner, end: Option<lexer::Token>) -> Vec<Expression> {
        let mut output: Vec<Expression> = Vec::new();
        loop {
            print_verbose!(">> ps {:?} {}", tokens.get_next(), tokens.current_idx());

            if tokens.current_is(&end) |
                tokens.current_is(&None) {
                // print_verbose!(">> ps EXIT {:?}", output);
                // tokens.inc();
                break;
            } else if tokens.current_is(&Some(lexer::Token::_NewLine)) {
                tokens.inc();
                continue;
            }

            let mut exp = Expression::new();
            exp.parse(tokens, Some(lexer::Token::Separator));
            print_verbose!("* {:?} {:?}", exp, tokens.get_token());
            if exp.elems.len()>0 {
                output.push(exp);
            }

        }
        output
    }

    fn _count_list_elems(tokens: &lexer::Scanner) -> i32 {
        let mut idx = tokens.current_idx();
        let mut elem_count: i32 = 0;
        let mut sub_scopes_count: i32 = 0;
        loop {
            let tkn = tokens.get_token_at(idx);
            match tkn {
                Some(t) => {
                    if *t == lexer::Token::ScopeStart('(') || *t == lexer::Token::ScopeStart('{') {
                        sub_scopes_count += 1;
                    }
                    else if sub_scopes_count > 0 {
                        if *t == lexer::Token::ScopeEnd(')') || *t == lexer::Token::ScopeEnd('}') {
                            sub_scopes_count -= 1;
                        }
                    }
                    else {
                        if *t == lexer::Token::ScopeEnd(')') || *t == lexer::Token::Separator {
                            elem_count += 1;
                            break;
                        }
                        else if *t == lexer::Token::List {
                            elem_count += 1;
                        }
                    }
                },
                None => break // end reached
            }
            idx += 1;
        }
        elem_count
    }

    fn parse(&mut self, tokens: &mut lexer::Scanner, end: Option<lexer::Token>) {
        // println!("-->");
        // println!("{:?}", tokens);
        loop {
            let tok = tokens.get_token();


            if tokens.current_is(&end) || tokens.current_is(&None) {
                tokens.inc();
                break; // If required end is reached, don't increment
            }

            let tok = tok.unwrap();
            print_verbose!("<Token> {:?}", tok);
            if let lexer::Token::_Comment = tok {
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
            let cat_tok = Obj::categorize(&tok);
            match cat_tok {
                Obj::Object(_) => {
                    self.elems.push(cat_tok);
                },
                Obj::Scope(s) => {
                    tokens.inc(); //skip over the scope start token
                    let exp_obj: Obj = match s {
                        '{' => {
                            let scoped = Expression::parse_scope(tokens, Some(lexer::Token::ScopeEnd('}')));
                            Obj::Group(scoped)
                        },
                        '(' => {
                            print_verbose!(">>>>>>>>>>> {:?} {:?}", &end, self);
                            let mut ex_list = Expression::new();
                            let elem_count = Expression::_count_list_elems(&tokens);

                            if elem_count == 1 {
                                // if only one elem, the syntax is like (a + 1) or (x)
                                // these are not considered list like
                                ex_list.parse(tokens, Some(lexer::Token::ScopeEnd(')')));
                            } else {
                                // parse each element
                                ex_list.op = Obj::Operator(lexer::Token::List);

                                for _ in 0..(elem_count - 1) {
                                    let mut ex = Expression::new();
                                    ex.parse(tokens, Some(lexer::Token::List));
                                    if ex.elems.len() > 0 {
                                        ex_list.elems.push(ex.to_object());
                                    }
                                }
                                let mut ex = Expression::new();
                                ex.parse(tokens, Some(lexer::Token::ScopeEnd(')')));
                                if ex.elems.len() > 0 {
                                    ex_list.elems.push(ex.to_object());
                                }
                            }

                            // current token will be ScopeEnd(')')
                            if tokens.next_is(&Some(lexer::Token::FuncDef)) { // hacky increment if this is part of funcdef
                                tokens.inc();
                            }
                            print_verbose!("<<<<<<<< {:?} {:?}", tokens.get_token(), ex_list);
                            ex_list.to_object()
                        },
                        _ => panic!("Illegal scope start char")
                    };

                    if let Some(nxt_t) = tokens.get_token() {
                        if let lexer::Token::FuncDef = *nxt_t { //Handle ()=>{} function definition
                            let mut exp = Expression::new();
                            exp.elems.push(exp_obj);
                            exp.op = Obj::Operator(lexer::Token::FuncDef);
                            tokens.inc(); // go to next token after the funcdef token
                            match tokens.get_token() {
                                None => panic!("Incomplete function definition"),
                                Some(t) => {
                                    match t {
                                        lexer::Token::ScopeStart('{') => {
                                            tokens.inc(); // move into scope
                                            let scoped = Expression::parse_scope(tokens, Some(lexer::Token::ScopeEnd('}')));
                                            print_verbose!("================{:?}", tokens.get_token());
                                            exp.elems.push(Obj::Group(scoped));
                                        },
                                        _ => {
                                            let mut body_exp = Expression::new();
                                            body_exp.parse(tokens, Some(lexer::Token::Separator));
                                            exp.elems.push(body_exp.to_object());
                                            // panic!("Single statements funcs not supported yet")
                                        }
                                    }
                                }
                            }
                            match exp.elems[1] {
                                Obj::Group(_) | Obj::Expr(_) => self.elems.push(exp.to_object()),
                                _ => panic!("Invalid function definition {:?}", exp) // func body should be Group or Expr
                            };
                            // break; // function definition complete
                        } else {
                            print_verbose!(" ++++++++++ {:?} {:?}", self, exp_obj);
                            self.elems.push(exp_obj);
                        }
                    } else {
                        self.elems.push(exp_obj);
                    }
                    continue; //skip final increment
                },


                Obj::Operator(op) => {
                    match (&self.op, &op) {

                        (Obj::Null, lexer::Token::FuncReturn) => { // return statement
                            // create new expressions for return statement
                            self.op = Obj::Operator(op);
                            tokens.inc(); // go to next token to parse return expression
                            let mut exp = Expression::new();
                            exp.parse(tokens, Some(lexer::Token::Separator)); // look for RHS
                            // flatten values of return statement
                            if exp.elems.len() == 1 {
                                self.elems.push(exp.elems[0].clone());
                            } else if exp.elems.len() > 1 {
                                self.elems.push(exp.to_object());
                            }
                            // println!(">> {:?}", self);
                            break; // return out of parse
                        },

                        (Obj::Null, _) => {
                            print_verbose!("New operator!!! {:?}", op);
                            self.op = Obj::Operator(op)
                        },

                        _ => { // fallback sequence
                            let mut exp = Expression::new();
                            if self.elems.len() > 0 {
                                exp.elems.push(self.elems.pop().unwrap()); // setup LHS
                            }
                            exp.parse(tokens, Some(lexer::Token::Separator)); // look for RHS
                            if exp.elems.len()!=0 {
                                self.elems.push(exp.to_object());
                            }
                            print_verbose!("-----------+ 2222 {:?} {:?}", tokens.get_token(), self);
                            break; //skip final increment
                        }
                    }
                },


                Obj::SuffixOperator(sop) => {
                    if self.elems.len() == 0 {
                        panic!("Suffix {} without symbol or expression", sop);
                    }
                    let mut exp = Expression::new();
                    exp.elems.push(self.elems.pop().unwrap()); // setup object to apply suffix operator to
                    exp.op = Obj::SuffixOperator(sop);
                    self.elems.push(exp.to_object());
                }
                _=>()
            }
            tokens.inc();
        }
        // println!("<--");
    }
}






pub fn parse(tokens: &mut lexer::Scanner) -> Vec<Expression> {
    print_verbose!("\n--------parsing start!--------");

    let output: Vec<Expression> = Expression::parse_scope(tokens, None);

    if env::var("VERBOSE").is_ok() {
        println!("\n--------parsing done!--------");
        for (i,o) in output.iter().enumerate() {
            println!("{} {:?}", i, o)
        }
        println!("------------------\n");
    }
    output
}
