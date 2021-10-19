
use std::fmt;

#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro

use crate::syntax::lexer;
use crate::lib::{funcdef, moddef};



#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Block {
    Null,
    Bool(bool),
    Object(lexer::Token),
    Operator(lexer::Token),
    Scope(char),
    Expr(Box<Expression>), // use Box since Expression has Block type members (recursive)
    List(Vec<Block>),
    Func(Box<funcdef::FuncDef>),
    FuncBody(Vec<Expression>),
    NativeFunc(funcdef::NativeFuncDef),
    Mod(moddef::Module),
    ModBody(Vec<Expression>), // same definition as FuncBody, but evaluated differently
}



impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Block::Object(o) => write!(f, "{}", o),
            Block::Operator(op) => write!(f, "{}", op),
            Block::Bool(b) => write!(f, "{}", b),
            Block::Null => write!(f, "null"),
            Block::List(l) => {
                write!(f, "(").unwrap();
                if l.len() > 0 {
                    for i in 0..(l.len()-1) {
                        write!(f, "{},", l[i]).unwrap();
                    };
                    write!(f, "{}", l[l.len()-1]).unwrap();
                }
                write!(f, ")")
            },
            Block::Mod(m) => write!(f, "<module at {:p}>", m),
            _ => write!(f, "{:?}", self),
        }
    }
}


impl Block {
    fn categorize(tok: &lexer::Token) -> Block {
        use lexer::Token;
        match tok {
            Token::Symbol(_)
            | Token::Number(_)
            | Token::Text(_)
                => Block::Object(tok.clone()),

            Token::Arith(_)
            | Token::Comparison(_)
            | Token::FuncDef
            | Token::Assign
            | Token::List
            | Token::FuncCall
            | Token::FuncReturn
            | Token::Index
            | Token::Accessor
                => Block::Operator(tok.clone()),

            Token::ScopeStart(s) => Block::Scope(*s),
            _ => Block::Null
        }
    }

    pub fn get_list(&self) -> Option<Vec<Block>> {
        match self {
            Block::List(l) => Some(l.clone()),
            _ => None,
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        match self {
            Block::Bool(b) => Some(*b),
            _ => None,
        }
    }
}




#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Expression {
    pub op: Block,
    pub elems: Vec<Block>,
}


impl Expression {

    fn new() -> Expression {
        Expression {
            op: Block::Null,
            elems: Vec::new(),
        }
    }

    pub fn to_object(mut self) -> Block {
        if self.op == Block::Null && self.elems.len() == 1 {
            self.elems.pop().unwrap()
        } else
        if let Block::Operator(lexer::Token::List) | Block::Null = self.op {
            Block::List(self.elems)
        } else
        if let Block::Operator(lexer::Token::FuncDef) = self.op {
            if self.elems.len() == 2 {
                let body = self.elems.pop().unwrap();
                let mut args = self.elems.pop().unwrap();
                // Block::Func(FuncDef {args, body})
                match args { // convert args to list
                    Block::List(_) => (),
                    _ => args = Block::List(vec![args])
                }
                match body {
                    Block::FuncBody(_) | Block::Expr(_) => (),
                    _ => panic!("Invalid function body")
                }
                Block::Func(Box::new(funcdef::FuncDef {args, body}))
            } else {
                panic!("Illegal function definition - {:?}", self)
            }
        }
        else {
            Block::Expr(Box::new(self))
        }
    }


    fn parse_scope(tokens: &mut lexer::TokenStream, end: Option<&[lexer::Token]>) -> Vec<Expression> {
        let mut output: Vec<Expression> = Vec::new();
        let mut end_tokens = end.unwrap_or(&[]).to_vec();
        end_tokens.push(lexer::Token::Separator);

        loop {
            print_verbose!(">> PARSE_SCOPE {:?} {:?} {}", end_tokens, tokens.get_next(), tokens.current_idx());

            if tokens.current_is_in(&end) |
                tokens.current_is(&None) {
                // print_verbose!(">> ps EXIT {:?}", output);
                tokens.inc();
                break;
            } else if tokens.current_is(&Some(lexer::Token::_NewLine)) {
                tokens.inc();
                continue;
            }

            let mut exp = Expression::new();
            print_verbose!("\nParsing Expression");
            print_verbose!("---    -----     ----  {:?} {:?}", tokens.get_current(), Some(end_tokens.as_slice()));
            exp.parse(tokens, Some(end_tokens.as_slice()));
            print_verbose!("---    -----     ----  {:?} {:?}", tokens.get_current(), Some(end_tokens.as_slice()));
            print_verbose!("* {:?} {:?}", exp, tokens.get_current());
            if exp.elems.len()>0 {
                output.push(exp);
            }

            if tokens.prev_is_in(&end) { // the exp.parse might increment over scope end before returning
                print_verbose!("\n\n ENDD {:?}", tokens.get_current());
                // tokens.inc(); /
                break;
            }

        }
        output
    }

    fn _count_list_elems(tokens: &lexer::TokenStream) -> i32 {
        let mut idx = tokens.current_idx();
        let mut elem_count: i32 = 0;
        let mut sub_scopes_count: i32 = 0;
        loop {
            let tkn = tokens.get_current_at(idx);
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

    fn _convert_to_child_elem(&mut self) {
        // make copy of current self, clear self and add the copy as first elem
        let exp = Expression {
            op: self.op.clone(),
            elems: self.elems.clone(),
        };
        self.elems.clear();
        self.elems.push(exp.to_object());
    }

    fn parse(&mut self, tokens: &mut lexer::TokenStream, end: Option<&[lexer::Token]>) {
        // println!("-->");
        // println!("{:?}", tokens);
        loop {
            let tok = tokens.get_current();

            if tokens.current_is_in(&end) || tokens.current_is(&None) {
                print_verbose!("\twant {:?} got {:?}", end, tokens.get_current());
                tokens.inc();
                break; // If required end is reached
            }

            let tok = tok.unwrap();
            print_verbose!("<Token> {:?}", tok);
            if let lexer::Token::_Comment = tok {
                loop { // loop till EOL
                    tokens.inc();
                    if let Some(t) = tokens.get_current() {
                        if t.is_newline_token() {
                            break;
                        }
                    }
                };
                tokens.inc();
                break;
            }
            let cat_tok = Block::categorize(&tok);
            match cat_tok {
                Block::Object(_) => {
                    self.elems.push(cat_tok);
                },
                Block::Scope(s) => {
                    tokens.inc(); //skip over the scope start token
                    let exp_obj: Block = match s {
                        '{' => {
                            let scoped = Expression::parse_scope(tokens, Some(&[lexer::Token::ScopeEnd('}')]));
                            Block::ModBody(scoped) // same definition as FuncBody, but evaluated differently
                        },
                        '[' => {
                            if let Block::Operator(lexer::Token::Index) = self.op {
                                let mut ex = Expression::new();
                                ex.parse(tokens, Some(&[lexer::Token::ScopeEnd(']')]));
                                ex.to_object()
                            } else {
                                panic!("Illegal use of [] operator");
                            }
                        },
                        '(' => {
                            print_verbose!(">>> SCOPETEST() {:?} {:?}", &end, self);
                            let mut ex_list = Expression::new();
                            let elem_count = Expression::_count_list_elems(&tokens);
                            print_verbose!("<list> {}", elem_count);
                            if elem_count == 1 {
                                // if only one elem, the syntax is like (a + 1) or (x)
                                // these are not considered list like
                                ex_list.parse(tokens, Some(&[lexer::Token::ScopeEnd(')')]));
                            } else {
                                // parse each element
                                ex_list.op = Block::Operator(lexer::Token::List);

                                for _ in 0..(elem_count - 1) {
                                    let mut ex = Expression::new();
                                    ex.parse(tokens, Some(&[lexer::Token::List]));
                                    if ex.elems.len() > 0 {
                                        ex_list.elems.push(ex.to_object());
                                    }
                                }
                                let mut ex = Expression::new();
                                ex.parse(tokens, Some(&[lexer::Token::ScopeEnd(')')]));
                                if ex.elems.len() > 0 {
                                    ex_list.elems.push(ex.to_object());
                                }
                            }

                            // current token will be ScopeEnd(')')
                            if tokens.next_is(&Some(lexer::Token::FuncDef)) { // hacky increment if this is part of funcdef
                                tokens.inc();
                            }
                            print_verbose!("<<< SCOPETEST() {:?} {:?}", tokens.get_current(), ex_list);
                            ex_list.to_object()
                        },
                        _ => panic!("Illegal scope start char")
                    };

                    if let Some(nxt_t) = tokens.get_current() {
                        if let lexer::Token::FuncDef = *nxt_t { //Handle ()=>{} function definition
                            let mut exp = Expression::new();
                            exp.elems.push(exp_obj);
                            exp.op = Block::Operator(lexer::Token::FuncDef);
                            tokens.inc(); // go to next token after the funcdef token
                            match tokens.get_current() {
                                None => panic!("Incomplete function definition"),
                                Some(t) => {
                                    match t {
                                        lexer::Token::ScopeStart('{') => {
                                            tokens.inc(); // move into scope
                                            print_verbose!("==========k1======{:?}", tokens.get_current());
                                            let scoped = Expression::parse_scope(tokens, Some(&[lexer::Token::ScopeEnd('}')]));
                                            print_verbose!("==========k2======{:?}", tokens.get_current());
                                            exp.elems.push(Block::FuncBody(scoped));
                                        },
                                        _ => {
                                            let mut body_exp = Expression::new();
                                            body_exp.parse(tokens, Some(&[lexer::Token::Separator]));
                                            exp.elems.push(body_exp.to_object());
                                            // panic!("Single statements funcs not supported yet")
                                        }
                                    }
                                }
                            }
                            match exp.elems[1] {
                                Block::FuncBody(_) | Block::Expr(_) => self.elems.push(exp.to_object()),
                                _ => panic!("Invalid function definition {:?}", exp) // func body should be FuncBody or Expr
                            };
                            // break; // function definition complete
                        } else {
                            self.elems.push(exp_obj);
                            print_verbose!(" ++++++++++ {:?}", self);
                        }
                    } else {
                        self.elems.push(exp_obj);
                    }
                    continue; //skip final increment
                },


                Block::Operator(op) => {
                    match (&self.op, &op) {

                        (Block::Null, lexer::Token::FuncReturn) => { // return statement
                            // create new expressions for return statement
                            self.op = Block::Operator(op);
                            tokens.inc(); // go to next token to parse return expression
                            let mut exp = Expression::new();
                            exp.parse(tokens, end); // look for RHS
                            // exp.parse(tokens, Some(&[lexer::Token::Separator])); // look for RHS
                            // flatten values of return statement
                            if exp.elems.len() == 1 {
                                self.elems.push(exp.elems[0].clone());
                            } else if exp.elems.len() > 1 {
                                self.elems.push(exp.to_object());
                            }
                            // println!(">> {:?}", self);
                            break; // return out of parse
                        },

                        (Block::Null, _) => {
                            print_verbose!("New operator!!! {:?}", op);
                            self.op = Block::Operator(op)
                        },

                        (_, lexer::Token::Index) => {
                            if self.elems.len() == 0 {
                                panic!("Suffix [] without symbol or expression");
                            }
                            self._convert_to_child_elem();
                            self.op = Block::Operator(op);
                        },

                        (Block::Operator(lexer::Token::FuncCall), lexer::Token::FuncCall) => {
                            // other cases where self.op is not FuncCall are automatically handled
                            if self.elems.len() == 0 {
                                panic!("Function call without symbol or expression");
                            }
                            self._convert_to_child_elem();
                            self.op = Block::Operator(op);
                        },

                        _ => { // fallback sequence
                            let mut exp = Expression::new();
                            if self.elems.len() > 0 {
                                exp.elems.push(self.elems.pop().unwrap()); // setup LHS
                            }
                            exp.parse(tokens, end); // look till end token reached
                            if exp.elems.len()!=0 {
                                self.elems.push(exp.to_object());
                            }
                            print_verbose!("-----------+ 2222 {:?} {:?}", tokens.get_current(), self);
                            break; //skip final increment
                        }
                    }
                },

                _=>()
            }
            tokens.inc();
        }
        // println!("<--");
    }
}






pub fn parse(tokens: &mut lexer::TokenStream) -> Vec<Expression> {
    print_verbose!("\n--------parsing start!--------");

    let output: Vec<Expression> = Expression::parse_scope(tokens, None);

    print_verbose!("\n--------parsing done!--------");
    print_verbose_iter!(output);
    print_verbose!("------------------\n");
    output
}
