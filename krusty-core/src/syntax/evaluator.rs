use std::collections::HashMap;
use std::path::{Component, PathBuf};
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait

#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro

use crate::syntax::parser::{Phrase, Expression};
use crate::syntax::lexer::Token;
use crate::lib::{moddef, builtins};



#[derive(Debug)]
pub struct NameSpace<'a> {
    builtin_funcs: Option<HashMap<String, Phrase>>,
    parent: Option<&'a NameSpace<'a>>,
    pub module: moddef::Module,
}


impl<'a> NameSpace<'a> {
    pub fn new(path: Option<&PathBuf>, parent: Option<&'a NameSpace<'a>>) -> NameSpace<'a> {
        let mut builtin_funcs: Option<HashMap<String, Phrase>> = None;
        if let None = parent {
            let mut b = HashMap::new();
            builtins::load(&mut b);
            builtin_funcs = Some(b);
        }
        NameSpace {
            module: moddef::Module::new(path),
            builtin_funcs,
            parent,
        }
    }

    pub fn to_object(self) -> Phrase {
        Phrase::Mod(self.module)
    }

    pub fn run(&mut self, elist: &Vec<Expression>) -> Phrase {
        let mut return_val: Phrase = Phrase::Null;
        for (_i, o) in elist.iter().enumerate() {
            return_val = self.solve_expr(o);
            if let Phrase::Operator(Token::FuncReturn) = o.op {
                if let None = self.parent {
                    panic!("cannot use return here!")
                } else {
                    return return_val
                }
            }
        }
        return_val
    }


    fn get(&self, key: &String) -> Option<Phrase> {
        match self.module.vars.get(key) {
            Some(v) => Some(v.clone()),
            None => {
                match self.parent {
                    Some(p) => p.get(key),
                    None => {
                        // No parent. Means we are at the top of the stack
                        // Search for builtins only at the top of the stack
                        match self.builtin_funcs.as_ref().expect("no builtins?!").get(key) {
                            Some(v) => Some(v.clone()),
                            None => {
                                panic!("Symbol '{}' not found", key)
                            }
                        }
                    }
                }

            },
        }
    }

    fn set(&mut self, key: String, value: Phrase) {
        self.module.vars.insert(key, value);
    }

    pub fn get_path(&self) -> Option<PathBuf> {
        if !self.module.path.is_none() { // path exists
            self.module.path.clone()
        } else if !self.parent.is_none() { // path doesn't exist, but parent exists
            self.parent.unwrap().get_path() // get from parent
        } else {
            None
        }
    }

    pub fn get_relative_path(&self, p: &String) -> PathBuf {
        let cur_path = self.get_path();
        match cur_path {
            Some(pbuf) => {
                let mut newbuf = pbuf.clone();
                if newbuf.is_dir() {
                    newbuf.push(PathBuf::from_slash(p)); // push new filename
                } else {
                    newbuf.set_file_name(PathBuf::from_slash(p)); // replace filename
                }
                 // this step ensures `..` is translated
                 // without this `..` fails on windows
                let mut out = PathBuf::new();
                for c in newbuf.components() {
                    match c {
                        Component::ParentDir => {out.pop();},
                        _ => out.push(c)
                    }
                }
                out
            },
            None => PathBuf::from_slash(p)
        }
    }

    pub fn resolve(&mut self, o: &Phrase) -> Phrase {
        match o {
            Phrase::Expr(ex) => self.solve_expr(ex),
            Phrase::Object(Token::Symbol(s)) => self.get(s).unwrap(),
            Phrase::List(l) => Phrase::List(l.into_iter().map(|x| self.resolve(x)).collect()),
            Phrase::ModBody(m) => {
                // resolve ModBody to Mod
                let mut ns = NameSpace::new(None, Some(self));
                ns.run(&m);
                ns.to_object()
            },
            Phrase::FuncBody(_) => Phrase::Null, // this should never be called I think
            _ => o.clone()
        }
    }

    fn assign(&mut self, key: &Phrase, value: &Phrase) {
        if let Phrase::Object(Token::Symbol(var)) = key {
            print_verbose!("assign {:?}", var);
            let val = self.resolve(value);
            self.set(var.to_string(), val);
        } else {
            panic!("LHS is not a valid symbol");
        }
    }


    fn solve_arith(&mut self, op: char, elems: &Vec<Phrase>) ->Result<Phrase, String> {
        let mut res: Option<f64> = None;//f64 = if "+-".contains(op) {0.0} else {1.0};

        for e in elems.iter() {
            let num = match self.resolve(e) {
                Phrase::Object(Token::Number(n)) => n,
                Phrase::List(l) if l.len()==1 => { // single element list - expressions enclosed in ()
                    match l[0] {
                        Phrase::Object(Token::Number(n)) => n,
                        _ => return Err(format!("Cannot perform Arith on {:?}", e))
                    }
                },
                _ => return Err(format!("Cannot perform Arith on {:?}", e))
            };

            print_verbose!("arith {:?} {} {}", res, op, num);
            res = match (res, op) {
                (Some(_a), '+') =>Some(_a+num),
                (Some(_a), '-') =>Some(_a-num),
                (Some(_a), '*') =>Some(_a*num),
                (Some(_a), '/') =>Some(_a/num),
                _ => Some(num)
            };
        }
        return Ok(Phrase::Object(Token::Number(res.expect("Arith error")))) //return
    }


    fn solve_comparison(&mut self, op: &String, elems: &Vec<Phrase>) ->Result<Phrase, String> {
        // this function uses Rust's PartialEq and PartialOrd to do comparison
        let vals: Vec<Phrase> = elems.iter().map(|x| self.resolve(x)).collect();
        // println!("{} ", builtins::_type(&vec![vals[0].clone()]) == builtins::_type(&vec![vals[1].clone()]));
        print_verbose!("compare {} {:?}", op, vals);
        match &op[..] {
            "==" => Ok(Phrase::Bool(vals[0]==vals[1])),
            "!=" => Ok(Phrase::Bool(vals[0]!=vals[1])),
            ">" => Ok(Phrase::Bool(vals[0]>vals[1])),
            "<" => Ok(Phrase::Bool(vals[0]<vals[1])),
            ">=" => Ok(Phrase::Bool(vals[0]>=vals[1])),
            "<=" => Ok(Phrase::Bool(vals[0]<=vals[1])),
            _ => Err("Unsupported operator".to_string())
        }
    }

    pub fn eval_func_obj(&mut self, func: &Phrase, args: &Phrase, name: Option<&String>) -> Phrase {
        let name = match name {
            Some(s) => s,
            None => "anonymous"
        };

        let args: Vec<Phrase> = match args {
            Phrase::List(l) => l.to_vec(),
            _ => vec![args.clone()]
        };

        match func {
            Phrase::Func(f) => {
                let req_args = f.args.get_list().expect("function definition error");
                if req_args.len() != args.len() {
                    panic!("function arguments for '{}' don't match", name);
                } else {
                    let mut exec_env = NameSpace::new(None, Some(self));
                    for (k,v) in req_args.iter().zip(args.iter()) {
                        exec_env.assign(&k, &v);
                    }
                    print_verbose!("CALL {} {:?}", name, f.body);
                    match &f.body { // return function result
                        Phrase::FuncBody(elist) => exec_env.run(&elist),
                        _ => panic!("function '{}' definition error", name),
                    }
                }
            },
            Phrase::NativeFunc(f) => {
                let clean_args: Vec<Phrase> = args.iter().map(|x| self.resolve(x)).collect();
                (f.func)(self, &clean_args)
            }
            _ => panic!("function '{}' definition error", name)
        }
    }

    fn eval_func(&mut self, name: &String, args: &Phrase) -> Phrase {
        // println!("<F> {:?}", args);
        match self.get(name) {
            None => panic!("function '{}' not defined"),
            Some(func) => {
                self.eval_func_obj(&func, args, Some(name))
            },
        }
    }

    fn pick_index(&self, idx: &Phrase, things: &Phrase) -> Phrase {
        // println!("{:?} [{:?}]", things, idx);
        match (idx, things) {
            (Phrase::Object(Token::Number(n)), Phrase::List(a)) => a[*n as usize].clone(),
            (Phrase::Object(Token::Number(n)), Phrase::Object(Token::Text(a))) => Phrase::Object(Token::Text(a.chars().nth(*n as usize).unwrap().to_string())),
            _ => panic!("cannot index {:?} with {:?}", things, idx)
        }
        // Phrase::Null
    }

    fn solve_expr(&mut self, exp: &Expression) -> Phrase {
        // println!("<E> {:?}", exp);
        match &exp.op {
            Phrase::Operator(Token::Assign) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    panic!("Illegal assignment");
                }
                self.assign(&exp.elems[0], &exp.elems[1]);
                Phrase::Null
            },
            Phrase::Operator(Token::Arith(op)) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    panic!("Illegal arithmetic operation");
                }
                match self.solve_arith(*op, &exp.elems) {
                    Ok(res) => res,
                    Err(e) => panic!("{}", e)
                } // return
            },
            Phrase::Operator(Token::Comparison(op)) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    panic!("Illegal comparison operation");
                }
                match self.solve_comparison(op, &exp.elems) {
                    Ok(res) => res,
                    Err(e) => panic!("{}", e)
                } // return
            },
            Phrase::Operator(Token::FuncCall) => {
                match &exp.elems[0] {
                    Phrase::Object(Token::Symbol(func_name)) => self.eval_func(func_name, &exp.elems[1]),
                    Phrase::Func(_) => self.eval_func_obj(&exp.elems[0], &exp.elems[1], None),
                    Phrase::Expr(ex) => {
                        let func = self.solve_expr(&ex);
                        self.eval_func_obj(&func, &exp.elems[1], None)
                    },
                    _ => Phrase::Null,
                }
            },
            Phrase::Operator(Token::FuncReturn) => { // will only return a list type object??
                let ret_list: Vec<Phrase> = exp.elems.iter().map(|e| self.resolve(e)).collect();
                match ret_list.len() {
                    // We also unwrap these late evaluated lists in case it has 0 or 1 elements
                    0 => Phrase::Null,
                    1 => ret_list[0].clone(),
                    _ => Phrase::List(ret_list)
                }
            },
            Phrase::Operator(Token::List) => {
                // These are some List type Expressions still unconverted to Phrase::List
                // They are usually deep inside a function definition needing late evaluation
                let ret_list: Vec<Phrase> = exp.elems.iter().map(|e| self.resolve(e)).collect();
                match ret_list.len() {
                    // We also unwrap these late evaluated lists in case it has 0 or 1 elements
                    0 => Phrase::Null,
                    1 => ret_list[0].clone(),
                    _ => Phrase::List(ret_list)
                }
            },
            Phrase::Operator(Token::Index) => {
                if exp.elems.len() != 2 {
                    panic!("Illegal index operation");
                }
                let val = self.resolve(&exp.elems[0]);
                let idx = self.resolve(&exp.elems[1]);
                self.pick_index(&idx, &val)
            },
            Phrase::Operator(Token::Accessor) => {
                if exp.elems.len() != 2 {
                    panic!("Illegal access operation");
                }
                match self.resolve(&exp.elems[0]) {
                    Phrase::Mod(m) => {
                        match &exp.elems[1] {
                            Phrase::Object(Token::Symbol(s)) => m.vars.get(s).expect("member not found").clone(),
                            Phrase::Expr(x) => {
                                match x.op {
                                    Phrase::Operator(Token::Assign) => panic!("cannot assign into module"),
                                    _ => {
                                        let mut ns = NameSpace { // create new execution namespace
                                            builtin_funcs: None,
                                            module: m,
                                            parent: Some(&self)
                                        };
                                        ns.solve_expr(x)
                                    }
                                }
                            },
                            _ => panic!("invalid use of '.' accessor")
                        }
                    },
                    _ => panic!("invalid use of '.' accessor")
                }
            }
            _ => exp.clone().to_object()
        }
    }
}
