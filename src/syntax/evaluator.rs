use std::collections::HashMap;

use crate::syntax::parser::{Obj, Expression, ExprList};
use crate::syntax::lexer::Token;


fn _arith_operate(a:f64, b:f64, op:char) -> f64 {
    match op {
        '+' => a+b,
        '-' => a-b,
        '*' => a*b,
        '/' => a/b,
        _ => a
    }
}


#[derive(Debug)]
pub struct Env<'a> {
    vars: HashMap<String, Obj>,
    parent: Option<&'a Env<'a>>,
}

impl<'a> Env<'a> {
    pub fn new(parent: Option<&'a Env<'a>>) -> Env<'a> {
        Env {
            vars: HashMap::new(),
            parent
        }
    }

    pub fn disperse(&mut self, elist: &'a ExprList) {
        for (i, o) in elist.exprs.iter().enumerate() {
            println!("=======");
            self.solve_expr(o);
            println!("{} {:?}=>>", i, self);
        }
        // let vo: Vec<Obj> = Vec::new();
    }

    fn get(&self, key: &String) -> Option<Obj> {
        match self.vars.get(key) {
            Some(v) => Some(v.clone()),
            None => {
                match self.parent {
                    Some(p) => p.get(key),
                    None => panic!("Symbol '{}' not found", key)
                }
            }
        }
    }

    fn assign(&mut self, key: &'a Obj, value: &'a Obj) {
        if let Obj::Object(Token::Symbol(var)) = key {
            println!("{:?}", var);
            match value {
                Obj::Object(Token::Number(_)) | Obj::Object(Token::Text(_)) => {
                    self.vars.insert(var.to_string(), value.clone());
                },
                Obj::Object(Token::Symbol(s)) => {
                    let mo = self.get(s).unwrap();
                    self.vars.insert(var.to_string(), mo);
                },
                Obj::Expr(x) => {
                    let res = self.solve_expr(x);
                    self.vars.insert(var.to_string(), res);
                },
                Obj::Group(gx) => {
                    if gx.exprs.len()==1 {
                        let res = self.solve_expr(&gx.exprs[0]);
                        self.vars.insert(var.to_string(), res);
                    } else {
                        self.vars.insert(var.to_string(), Obj::Null);
                    }
                    // match self.solve_expr(x) {
                    //     Some(o) => {self.vars.insert(var.to_string(), o.clone());},
                    //     None => {self.vars.insert(var.to_string(), Obj::Null);}
                    // }
                },
                _ => {self.vars.insert(var.to_string(), Obj::Null);},
            };
            //
        } else {
            panic!("LHS is not a valid symbol");
        }
    }


    fn solve_arith(&mut self, op: char, elems: &'a Vec<Obj>) ->Result<Obj, String> {
        // elems should have only 2 members
        if elems.len() > 2 {
            return Err("Illegal arithmetic operation".to_string());
        }
        let mut res: f64 = if "+-".contains(op) {0.0} else {1.0};
        for e in elems.iter() {
            println!("{:?} {:?}", op, e);
            match e {
                Obj::Object(Token::Number(n)) => {
                    res = _arith_operate(res, *n, op);
                },
                Obj::Expr(x) => {
                    match self.solve_expr(x) {
                        Obj::Object(Token::Number(n)) => {
                            res = _arith_operate(res, n, op);
                        },
                        _ => {
                            return Err(format!("Cannot perform Arith {:?}", x));
                        }
                    };
                }
                _ => ()
            }
        }
        return Ok(Obj::Object(Token::Number(res))) //return
    }


    fn solve_expr(&mut self, exp: &'a Expression) -> Obj {
        // println!("{:?}", exp);
        match exp.op {
            Obj::Operator(Token::Arith(op)) => {
                match self.solve_arith(op, &exp.elems) {
                    Ok(res) => res,
                    Err(e) => panic!("{}", e)
                } // return
            },
            Obj::Operator(Token::Assign) => {
                // elems should have only 2 members
                if exp.elems.len() > 2 {
                    panic!("Illegal assignment");
                }
                self.assign(&exp.elems[0], &exp.elems[1]);
                Obj::Null
            },
            // Obj::Operator(Token::List) => {

            // },
            _ => Obj::Null
        }
    }
}
