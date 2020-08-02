use std::collections::HashMap;

use crate::syntax::parser::{Obj, Expression, ExprList};
use crate::syntax::lexer::Token;


fn _arith_operate(a:Option<f64>, b:f64, op:char) -> Option<f64> {

    println!("arith {:?} {} {}", a, op, b);
    match (a, op) {
        (Some(_a), '+') =>Some(_a+b),
        (Some(_a), '-') =>Some(_a-b),
        (Some(_a), '*') =>Some(_a*b),
        (Some(_a), '/') =>Some(_a/b),
        _ => Some(b)
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

    pub fn run(&mut self, elist: &'a ExprList) {
        for (_i, o) in elist.exprs.iter().enumerate() {
            // println!("=======");
            self.solve_expr(o);
            // println!("{} {:?}=>>", i, self);
            // println!("{} =>>", i);
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
            println!("assign {:?}", var);
            match value {
                Obj::Object(Token::Number(_)) | Obj::Object(Token::Text(_)) => {
                    self.vars.insert(var.to_string(), value.clone());
                },
                Obj::Object(Token::Symbol(s)) => {
                    let mo = self.get(s).unwrap();
                    self.vars.insert(var.to_string(), mo);
                },
                Obj::Func(_) => {
                    self.vars.insert(var.to_string(), value.clone());
                },
                Obj::List(l) => {
                    let solved_list = l.into_iter().map(|x| {
                        match x {
                            Obj::Expr(ex) => self.solve_expr(ex),
                            Obj::Object(Token::Symbol(s)) => self.get(s).unwrap(),
                            _ => x.clone()
                        }
                    }).collect();
                    self.vars.insert(var.to_string(), Obj::List(solved_list));
                },
                Obj::Expr(x) => {
                    let res = self.solve_expr(x);
                    self.vars.insert(var.to_string(), res);
                },
                Obj::Group(_) => {
                    self.vars.insert(var.to_string(), Obj::Null);
                },
                _ => panic!("Illegal assignment - {:?}", value),
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
        let mut res: Option<f64> = None;//f64 = if "+-".contains(op) {0.0} else {1.0};
        for e in elems.iter() {
            // println!("{:?} {:?}", op, e);
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
                            return Err(format!("Cannot perform Arith on {:?}", x));
                        }
                    };
                }
                _ => return Err(format!("Cannot perform Arith on {:?}", e))
            }
        }
        return Ok(Obj::Object(Token::Number(res.expect("Arith error")))) //return
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
                if exp.elems.len() != 2 {
                    panic!("Illegal assignment");
                }
                self.assign(&exp.elems[0], &exp.elems[1]);
                Obj::Null
            },
            Obj::Operator(Token::FuncCall) => {
                if let Obj::Object(Token::Symbol(func_name)) = &exp.elems[0] {
                    self.eval_func(func_name, &exp.elems[1]);
                }
                Obj::Null
                // let args = exp.elems[1]
            }
            _ => Obj::Null
        }
    }

    fn eval_func(&mut self, name: &String, args: &'a Obj) {
        let args = match args {
            Obj::Expr(e) => Obj::List(vec![self.solve_expr(e)]),
            _ => args.clone()
        };
        match self.get(name) {
            None => panic!("function '{}' not defined"),
            Some(func) => {
                let mut exec_env = Env::new(Some(self));

                println!("CALL {} {} {:?}", name, func, args)
            },
        };
    }
}
