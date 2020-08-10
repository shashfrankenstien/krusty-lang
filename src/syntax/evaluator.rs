use std::collections::HashMap;
use std::env;

use crate::syntax::parser::{Obj, Expression, ExprList};
use crate::syntax::lexer::Token;
use crate::lib::builtins;



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

    pub fn run(&mut self, elist: &ExprList) -> Obj {
        let mut return_val = Obj::Null;
        for (_i, o) in elist.exprs.iter().enumerate() {
            return_val = self.solve_expr(o);
        }
        return_val
    }


    fn get(&self, key: &String) -> Option<Obj> {
        match self.vars.get(key) {
            Some(v) => Some(v.clone()),
            None => {
                match self.builtin_funcs.get(key) {
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

    fn resolve(&mut self, o: &Obj) -> Obj {
        match o {
            Obj::Expr(ex) => self.solve_expr(ex),
            Obj::Object(Token::Symbol(s)) => self.get(s).unwrap(),
            Obj::List(l) => Obj::List(l.into_iter().map(|x| self.resolve(x)).collect()),
            Obj::Group(_) => Obj::Null, // TODO
            _ => o.clone()
        }
    }

    fn assign(&mut self, key: &Obj, value: &Obj) {
        if let Obj::Object(Token::Symbol(var)) = key {
            if env::var("VERBOSE").is_ok() {
                println!("assign {:?}", var);
            }
            let val = self.resolve(value);
            self.set(var.to_string(), val);
        } else {
            panic!("LHS is not a valid symbol");
        }
    }


    fn solve_arith(&mut self, op: char, elems: &Vec<Obj>) ->Result<Obj, String> {
        let mut res: Option<f64> = None;//f64 = if "+-".contains(op) {0.0} else {1.0};
        for e in elems.iter() {
            match self.resolve(e) {
                Obj::Object(Token::Number(n)) => {
                    if env::var("VERBOSE").is_ok() {
                        println!("arith {:?} {} {}", res, op, n);
                    }
                    res = match (res, op) {
                        (Some(_a), '+') =>Some(_a+n),
                        (Some(_a), '-') =>Some(_a-n),
                        (Some(_a), '*') =>Some(_a*n),
                        (Some(_a), '/') =>Some(_a/n),
                        _ => Some(n)
                    };
                },
                _ => return Err(format!("Cannot perform Arith on {:?}", e))
            }
        }
        return Ok(Obj::Object(Token::Number(res.expect("Arith error")))) //return
    }


    fn solve_comparison(&mut self, op: &String, elems: &Vec<Obj>) ->Result<Obj, String> {
        // this function uses Rust's PartialEq and PartialOrd to do comparison
        let vals: Vec<Obj> = elems.iter().map(|x| self.resolve(x)).collect();
        // println!("{} ", builtins::_type(&vec![vals[0].clone()]) == builtins::_type(&vec![vals[1].clone()]));
        if env::var("VERBOSE").is_ok() {
            println!("compare {} {:?}", op, vals);
        }
        match &op[..] {
            "==" => Ok(Obj::Bool(vals[0]==vals[1])),
            "!=" => Ok(Obj::Bool(vals[0]!=vals[1])),
            ">" => Ok(Obj::Bool(vals[0]>vals[1])),
            "<" => Ok(Obj::Bool(vals[0]<vals[1])),
            ">=" => Ok(Obj::Bool(vals[0]>=vals[1])),
            "<=" => Ok(Obj::Bool(vals[0]<=vals[1])),
            _ => Err("Unsupported operator".to_string())
        }
    }


    fn eval_func(&mut self, name: &String, args: &Obj) -> Obj {
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
                        let f = builtins::find_func(&f[..]);
                        let clean_args: Vec<Obj> = args.iter().map(|x| self.resolve(x)).collect();
                        f(&clean_args)
                    }
                    _ => panic!("function '{}' definition error", name)
                }
            },
        }
    }

    fn pick_index(&self, idx: &Token, things: &Obj) -> Obj {
        // println!("{:?} [{:?}]", things, idx);
        match (idx, things) {
            (Token::Number(n), Obj::List(a)) => a[*n as usize].clone(),
            (Token::Number(n), Obj::Object(Token::Text(a))) => Obj::Object(Token::Text(a.chars().nth(*n as usize).unwrap().to_string())),
            _ => panic!("cannot index {:?} with {:?}", things, idx)
        }
        // Obj::Null
    }

    fn solve_expr(&mut self, exp: &Expression) -> Obj {
        // println!("{:?}", exp);
        match &exp.op {
            Obj::Operator(Token::Assign) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    panic!("Illegal assignment");
                }
                self.assign(&exp.elems[0], &exp.elems[1]);
                Obj::Null
            },
            Obj::Operator(Token::Arith(op)) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    panic!("Illegal arithmetic operation");
                }
                match self.solve_arith(*op, &exp.elems) {
                    Ok(res) => res,
                    Err(e) => panic!("{}", e)
                } // return
            },
            Obj::Operator(Token::Comparison(op)) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    panic!("Illegal comparison operation");
                }
                match self.solve_comparison(op, &exp.elems) {
                    Ok(res) => res,
                    Err(e) => panic!("{}", e)
                } // return
            },
            Obj::Operator(Token::FuncCall) => {
                match &exp.elems[0] {
                    Obj::Object(Token::Symbol(func_name)) => self.eval_func(func_name, &exp.elems[1]),
                    _ => Obj::Null,
                }
            },
            Obj::Operator(Token::List) => {
                // These are some List type Expressions still unconverted to Obj::List
                // They are usually deep inside a function definition needing late evaluation
                let ret_list: Vec<Obj> = exp.elems.iter().map(|e| self.resolve(e)).collect();
                match ret_list.len() {
                    // We also unwrap these late evaluated lists in case it has 0 or 1 elements
                    0 => Obj::Null,
                    1 => ret_list[0].clone(),
                    _ => Obj::List(ret_list)
                }
            },
            Obj::SuffixOperator(Token::Index(idx)) => {
                if exp.elems.len() != 1 {
                    panic!("Illegal index operation");
                }
                let val = self.resolve(&exp.elems[0]);
                self.pick_index(idx, &val)
            }
            _ => exp.clone().to_object()
        }
    }
}
