use std::collections::HashMap;
use std::path::{Component, PathBuf};
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait

#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro

use crate::syntax::parser::{Block, Expression};
use crate::syntax::lexer::Token;
use crate::lib::{moddef, builtins};



#[derive(Debug)]
pub struct NameSpace<'a> {
    builtin_funcs: Option<HashMap<String, Block>>,
    parent: Option<&'a NameSpace<'a>>,
    pub module: moddef::Module,
}


impl<'a> NameSpace<'a> {
    pub fn new(path: Option<&PathBuf>, parent: Option<&'a NameSpace<'a>>) -> NameSpace<'a> {
        let mut builtin_funcs: Option<HashMap<String, Block>> = None;
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

    pub fn to_object(self) -> Block {
        Block::Mod(self.module)
    }

    pub fn run(&mut self, elist: &Vec<Expression>) -> Block {
        let mut return_val: Block = Block::Null;
        for (_i, o) in elist.iter().enumerate() {
            return_val = self.solve_expr(o);
            if let Block::Operator(Token::FuncReturn) = o.op {
                if let None = self.parent {
                    panic!("cannot use return here!")
                } else {
                    return return_val
                }
            }
        }
        return_val
    }


    fn get(&self, key: &String) -> Option<Block> {
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

    fn set(&mut self, key: String, value: Block) {
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

    pub fn resolve(&mut self, o: &Block) -> Block {
        match o {
            Block::Expr(ex) => self.solve_expr(ex),
            Block::Object(Token::Symbol(s)) => self.get(s).unwrap(),
            Block::List(l) => Block::List(l.into_iter().map(|x| self.resolve(x)).collect()),
            Block::ModBody(m) => {
                // resolve ModBody to Mod
                let mut ns = NameSpace::new(None, Some(self));
                ns.run(&m);
                ns.to_object()
            },
            Block::FuncBody(_) => Block::Null, // this should never be called I think
            _ => o.clone()
        }
    }

    fn assign(&mut self, key: &Block, value: &Block) {
        if let Block::Object(Token::Symbol(var)) = key {
            print_verbose!("assign {:?}", var);
            let val = self.resolve(value);
            self.set(var.to_string(), val);
        } else {
            panic!("LHS is not a valid symbol");
        }
    }


    fn solve_arith(&mut self, op: char, elems: &Vec<Block>) ->Result<Block, String> {
        let mut res: Option<f64> = None;//f64 = if "+-".contains(op) {0.0} else {1.0};

        for e in elems.iter() {
            let num = match self.resolve(e) {
                Block::Object(Token::Number(n)) => n,
                Block::List(l) if l.len()==1 => { // single element list - expressions enclosed in ()
                    match l[0] {
                        Block::Object(Token::Number(n)) => n,
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
        return Ok(Block::Object(Token::Number(res.expect("Arith error")))) //return
    }


    fn solve_comparison(&mut self, op: &String, elems: &Vec<Block>) ->Result<Block, String> {
        // this function uses Rust's PartialEq and PartialOrd to do comparison
        let vals: Vec<Block> = elems.iter().map(|x| self.resolve(x)).collect();
        // println!("{} ", builtins::_type(&vec![vals[0].clone()]) == builtins::_type(&vec![vals[1].clone()]));
        print_verbose!("compare {} {:?}", op, vals);
        match &op[..] {
            "==" => Ok(Block::Bool(vals[0]==vals[1])),
            "!=" => Ok(Block::Bool(vals[0]!=vals[1])),
            ">" => Ok(Block::Bool(vals[0]>vals[1])),
            "<" => Ok(Block::Bool(vals[0]<vals[1])),
            ">=" => Ok(Block::Bool(vals[0]>=vals[1])),
            "<=" => Ok(Block::Bool(vals[0]<=vals[1])),
            _ => Err("Unsupported operator".to_string())
        }
    }

    pub fn eval_func_obj(&mut self, func: &Block, args: &Block, name: Option<&String>) -> Block {
        let name = match name {
            Some(s) => s,
            None => "anonymous"
        };

        let args: Vec<Block> = match args {
            Block::List(l) => l.to_vec(),
            _ => vec![args.clone()]
        };

        match func {
            Block::Func(f) => {
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
                        Block::FuncBody(elist) => exec_env.run(&elist),
                        _ => panic!("function '{}' definition error", name),
                    }
                }
            },
            Block::NativeFunc(f) => {
                let clean_args: Vec<Block> = args.iter().map(|x| self.resolve(x)).collect();
                (f.func)(self, &clean_args)
            }
            _ => panic!("function '{}' definition error", name)
        }
    }

    fn eval_func(&mut self, name: &String, args: &Block) -> Block {
        // println!("<F> {:?}", args);
        match self.get(name) {
            None => panic!("function '{}' not defined"),
            Some(func) => {
                self.eval_func_obj(&func, args, Some(name))
            },
        }
    }

    fn pick_index(&self, idx: &Block, things: &Block) -> Block {
        // println!("{:?} [{:?}]", things, idx);
        match (idx, things) {
            (Block::Object(Token::Number(n)), Block::List(a)) => a[*n as usize].clone(),
            (Block::Object(Token::Number(n)), Block::Object(Token::Text(a))) => Block::Object(Token::Text(a.chars().nth(*n as usize).unwrap().to_string())),
            _ => panic!("cannot index {:?} with {:?}", things, idx)
        }
        // Block::Null
    }

    fn solve_expr(&mut self, exp: &Expression) -> Block {
        // println!("<E> {:?}", exp);
        match &exp.op {
            Block::Operator(Token::Assign) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    panic!("Illegal assignment");
                }
                self.assign(&exp.elems[0], &exp.elems[1]);
                Block::Null
            },
            Block::Operator(Token::Arith(op)) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    panic!("Illegal arithmetic operation");
                }
                match self.solve_arith(*op, &exp.elems) {
                    Ok(res) => res,
                    Err(e) => panic!("{}", e)
                } // return
            },
            Block::Operator(Token::Comparison(op)) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    panic!("Illegal comparison operation");
                }
                match self.solve_comparison(op, &exp.elems) {
                    Ok(res) => res,
                    Err(e) => panic!("{}", e)
                } // return
            },
            Block::Operator(Token::FuncCall) => {
                match &exp.elems[0] {
                    Block::Object(Token::Symbol(func_name)) => self.eval_func(func_name, &exp.elems[1]),
                    Block::Func(_) => self.eval_func_obj(&exp.elems[0], &exp.elems[1], None),
                    Block::Expr(ex) => {
                        let func = self.solve_expr(&ex);
                        self.eval_func_obj(&func, &exp.elems[1], None)
                    },
                    _ => Block::Null,
                }
            },
            Block::Operator(Token::FuncReturn) => { // will only return a list type object??
                let ret_list: Vec<Block> = exp.elems.iter().map(|e| self.resolve(e)).collect();
                match ret_list.len() {
                    // We also unwrap these late evaluated lists in case it has 0 or 1 elements
                    0 => Block::Null,
                    1 => ret_list[0].clone(),
                    _ => Block::List(ret_list)
                }
            },
            Block::Operator(Token::List) => {
                // These are some List type Expressions still unconverted to Block::List
                // They are usually deep inside a function definition needing late evaluation
                let ret_list: Vec<Block> = exp.elems.iter().map(|e| self.resolve(e)).collect();
                match ret_list.len() {
                    // We also unwrap these late evaluated lists in case it has 0 or 1 elements
                    0 => Block::Null,
                    1 => ret_list[0].clone(),
                    _ => Block::List(ret_list)
                }
            },
            Block::Operator(Token::Index) => {
                if exp.elems.len() != 2 {
                    panic!("Illegal index operation");
                }
                let val = self.resolve(&exp.elems[0]);
                let idx = self.resolve(&exp.elems[1]);
                self.pick_index(&idx, &val)
            },
            Block::Operator(Token::Accessor) => {
                if exp.elems.len() != 2 {
                    panic!("Illegal access operation");
                }
                match self.resolve(&exp.elems[0]) {
                    Block::Mod(m) => {
                        match &exp.elems[1] {
                            Block::Object(Token::Symbol(s)) => m.vars.get(s).expect("member not found").clone(),
                            Block::Expr(x) => {
                                match x.op {
                                    Block::Operator(Token::Assign) => panic!("cannot assign into module"),
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
