use std::collections::HashMap;
use std::env;

use crate::syntax::parser::{Obj, Expression, ExprList};
use crate::syntax::lexer::Token;
use crate::lib::builtins;

fn _arith_operate(a:Option<f64>, b:f64, op:char) -> Option<f64> {
    if env::var("VERBOSE").is_ok() {
        println!("arith {:?} {} {}", a, op, b);
    }
    match (a, op) {
        (Some(_a), '+') =>Some(_a+b),
        (Some(_a), '-') =>Some(_a-b),
        (Some(_a), '*') =>Some(_a*b),
        (Some(_a), '/') =>Some(_a/b),
        _ => Some(b)
    }
}


#[derive(Debug)]
pub struct NameSpace<'a> {
    builtin_funcs: HashMap<String, Obj>,
    vars: HashMap<String, Obj>,
    parent: Option<&'a NameSpace<'a>>,
}

impl<'a> NameSpace<'a> {
    pub fn new(parent: Option<&'a NameSpace<'a>>) -> NameSpace<'a> {
        let mut builtin_funcs = HashMap::new();
        if let None = parent {
            builtins::load(&mut builtin_funcs);
        }
        NameSpace {
            vars: HashMap::new(),
            builtin_funcs,
            parent
        }
    }

    pub fn run(&mut self, elist: &'a ExprList) -> Obj {
        let mut return_val = Obj::Null;
        for (_i, o) in elist.exprs.iter().enumerate() {
            return_val = self.solve_expr(o);
        }
        return_val
    }


    fn get(&self, key: &String) -> Option<Obj> {
        match self.builtin_funcs.get(key) {
            Some(v) => Some(v.clone()),
            None => {
                match self.vars.get(key) {
                    Some(v) => Some(v.clone()),
                    None => {
                        match self.parent {
                            Some(p) => p.get(key),
                            None => panic!("Symbol '{}' not found", key)
                        }
                    }
                }
            },
        }
    }

    fn set(&mut self, key: String, value: Obj) {
        self.vars.insert(key, value);
    }

    fn assign(&mut self, key: &'a Obj, value: &'a Obj) {
        if let Obj::Object(Token::Symbol(var)) = key {
            if env::var("VERBOSE").is_ok() {
                println!("assign {:?}", var);
            }
            match value {
                Obj::Object(Token::Number(_)) | Obj::Object(Token::Text(_)) => {
                    self.set(var.to_string(), value.clone());
                },
                Obj::Object(Token::Symbol(s)) => {
                    let mo = self.get(s).unwrap();
                    self.set(var.to_string(), mo);
                },
                Obj::Func(_) => {
                    self.set(var.to_string(), value.clone());
                },
                Obj::List(l) => {
                    let solved_list = l.into_iter().map(|x| {
                        match x {
                            Obj::Expr(ex) => self.solve_expr(ex),
                            Obj::Object(Token::Symbol(s)) => self.get(s).unwrap(),
                            _ => x.clone()
                        }
                    }).collect();
                    self.set(var.to_string(), Obj::List(solved_list));
                },
                Obj::Expr(x) => {
                    let res = self.solve_expr(x);
                    self.set(var.to_string(), res);
                },
                Obj::Group(_) => {
                    self.set(var.to_string(), Obj::Null);
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
                Obj::Object(Token::Symbol(s)) => {
                    let val = self.get(s).unwrap();
                    match val {
                        Obj::Object(Token::Number(n)) => {
                            res = _arith_operate(res, n, op);
                        },
                        _ => {return Err(format!("Cannot perform Arith on {:?}", e));}
                    }
                },
                Obj::Expr(x) => {
                    match self.solve_expr(x) {
                        Obj::Object(Token::Number(n)) => {
                            res = _arith_operate(res, n, op);
                        },
                        _ => {return Err(format!("Cannot perform Arith on {:?}", x));}
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
                match &exp.elems[0] {
                    Obj::Object(Token::Symbol(func_name)) => self.eval_func(func_name, &exp.elems[1]),
                    _ => Obj::Null,
                }
            },
            Obj::Operator(Token::List) => { // These are some List type Operators still unconverted to Obj::List
                let ret_list: Vec<Obj> = exp.elems.iter().map(|e| {
                    match e {
                        Obj::Expr(ex) => self.solve_expr(ex),
                        Obj::Object(Token::Symbol(s)) => self.get(s).expect("Cannot find variable"),
                        _ => e.clone()
                    }
                }).collect();
                match ret_list.len() {
                    0 => Obj::Null,
                    1 => ret_list[0].clone(),
                    _ => Obj::List(ret_list)
                }
            }
            _ => exp.clone().to_object()
        }
    }

    fn eval_func(&mut self, name: &String, args: &'a Obj) -> Obj {
        let args = match args {
            Obj::Expr(e) => Obj::List(vec![self.solve_expr(e)]),
            _ => args.clone()
        };
        let args = args.get_list().expect("function arguments should be of internal type Obj::List");
        match self.get(name) {
            None => panic!("function '{}' not defined"),
            Some(func) => {
                match func {
                    Obj::Func(f) => {
                        let req_args = f.args.get_list().expect("function definition error");
                        if req_args.len() != args.len() {
                            panic!("function arguments for '{}' don't match", name);
                        } else {
                            let mut exec_env = NameSpace::new(Some(self));
                            for (k,v) in req_args.iter().zip(args.iter()) {
                                exec_env.assign(&k, &v);
                            }
                            if env::var("VERBOSE").is_ok() {
                                println!("CALL {} {:?}", name, f.body);
                            }
                            match f.body { // return function result
                                Obj::Group(elist) => exec_env.run(&elist),
                                _ => panic!("function '{}' definition error", name),
                            }
                        }
                    },
                    Obj::BuiltinFunc(f) => {
                        let f = builtins::find(&f[..]);
                        let args: Vec<Obj> = args.iter().map(|x| {
                            match x {
                                // Obj::Expr(ex) => self.solve_expr(&ex),
                                Obj::Object(Token::Symbol(s)) => self.get(s).unwrap(),
                                _ => x.clone()
                            }
                        }).collect();
                        f(&args)
                    }
                    _ => panic!("function '{}' definition error", name)
                }
            },
        }
    }
}
