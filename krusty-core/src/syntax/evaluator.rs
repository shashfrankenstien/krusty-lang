use std::collections::HashMap;
use std::path::{Component, PathBuf};
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait

#[cfg(debug_assertions)]
use std::env; // required for print_verbose! macro

use crate::syntax::parser::{Block, Expression};
use crate::syntax::lexer::Token;
use crate::lib::{moddef, builtins};

use super::errors::{Error, KrustyErrorType};




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
            builtins::load_all(&mut b);
            builtin_funcs = Some(b);
        }
        NameSpace {
            module: moddef::Module::new(path),
            builtin_funcs,
            parent,
        }
    }


    pub fn to_block(self) -> Result<Block, KrustyErrorType> {
        Ok(Block::Mod(self.module))
    }

    pub fn run(&mut self, elist: &Vec<Expression>) -> Result<Block, KrustyErrorType> {
        let mut return_val: Block = Block::Null;
        for (_i, o) in elist.iter().enumerate() {
            return_val = self.solve_expr(o)?;
            if let Block::Operator(Token::FuncReturn) = o.op {
                if let None = self.parent {
                    eval_error!("cannot use return here!")
                } else {
                    return Ok(return_val)
                }
            }
        }
        Ok(return_val)
    }


    fn get(&self, key: &String) -> Result<Block, KrustyErrorType> {
        match self.module.vars.get(key) {
            Some(v) => Ok(v.clone()),
            None => {
                match self.parent {
                    Some(p) => p.get(key),
                    None => {
                        // No parent. Means we are at the top of the stack
                        // Search for builtins only at the top of the stack
                        match self.builtin_funcs.as_ref().expect("no builtins?!").get(key) {
                            Some(v) => Ok(v.clone()),
                            None => {
                                eval_error!(format!("Symbol '{}' not found", key))
                            }
                        }
                    }
                }

            },
        }
    }

    fn get_mut(&mut self, key: &String) -> Result<&mut Block, KrustyErrorType> {
        // for now, refs are mutable by default and can only be gotten from within the same scope
        match self.module.vars.get_mut(key) {
            Some(v) => Ok(v),
            None => eval_error!(format!("Symbol '{}' not found in scope", key))
        }
    }

    fn set(&mut self, key: String, value: Block) -> Result<(), KrustyErrorType> {
        self.module.vars.insert(key, value);
        Ok(())
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

    pub fn resolve(&mut self, o: &Block) -> Result<Block, KrustyErrorType> {
        match o {
            Block::Expr(ex) => self.solve_expr(ex),
            Block::Object(Token::Symbol(s)) => self.get(s),
            // Block::List(l) => Ok(Block::List(l.into_iter().map(|x| self.resolve(x)?).collect())),
            Block::List(l) => {
                let bl = NameSpace::resolve_vector(l, &mut |x| self.resolve(x))?;
                Ok(Block::List(bl))
            },
            Block::ModBody(m) => {
                // resolve ModBody to Mod
                let mut ns = NameSpace::new(None, Some(self));
                ns.run(&m)?;
                ns.to_block()
            },
            Block::FuncBody(_) => Ok(Block::Null), // this should never be called I think
            _ => Ok(o.clone())
        }
    }

    pub fn resolve_vector<I, F>(input: &Vec<I>, solver: &mut F) -> Result<Vec<Block>, KrustyErrorType>
        where F: FnMut(&I) -> Result<Block, KrustyErrorType>
    {
        let mut out: Vec<Block> = Vec::new();
        for i in input.into_iter() {
            let o = solver(i)?;
            out.push(o);
        }
        Ok(out)
    }


    fn assign(&mut self, key: &Block, value: &Block) -> Result<(), KrustyErrorType> {
        let val = self.resolve(value)?;
        match key {
            Block::Object(Token::Symbol(var)) => {
                print_verbose!("assign {:?}", var);
                self.set(var.to_string(), val)
            },
            Block::Expr(e) => {
                match &e.elems[0] {
                    Block::Object(Token::Symbol(k)) => {
                        let variable = self.get_mut(&k)?; // get mutable reference to variable so it can be modified inplace
                        match (variable, &e.op, &e.elems[1]) {
                            (l, Block::Operator(Token::Index), Block::Object(Token::Number(n))) => {
                                match l.update_list(*n as usize, val) {
                                    Ok(_) => Ok(()),
                                    Err(e) => eval_error!(format!("List assignment failed {}", e))
                                }
                            },
                            (Block::Mod(m), Block::Operator(Token::Accessor), Block::Object(Token::Symbol(prop))) => {
                                m.vars.insert(prop.to_string(), val);
                                Ok(())
                            },
                            _ => eval_error!("Unsupported assignment")
                        }
                    },
                    _ => eval_error!("Unsupported assignment")
                }

            }
            _ => {
                let k = self.resolve(key)?;
                println!("{:?}", key);
                println!("{:?}", k);
                eval_error!("LHS is not a valid symbol");
            }
        }
    }


    fn solve_arith(&mut self, op: char, elems: &Vec<Block>) ->Result<Block, KrustyErrorType> {
        let mut res: Option<f64> = None;//f64 = if "+-".contains(op) {0.0} else {1.0};

        for e in elems.iter() {
            let num = match self.resolve(e)? {
                Block::Object(Token::Number(n)) => n,
                Block::List(l) if l.len()==1 => { // single element list - expressions enclosed in ()
                    match l[0] {
                        Block::Object(Token::Number(n)) => n,
                        _ => eval_error!(format!("Cannot perform Arith on {:?}", e))
                    }
                },
                _ => eval_error!(format!("Cannot perform Arith on {:?}", e))
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


    fn solve_comparison(&mut self, op: &String, elems: &Vec<Block>) ->Result<Block, KrustyErrorType> {
        // this function uses Rust's PartialEq and PartialOrd to do comparison
        let vals: Vec<Block> = NameSpace::resolve_vector(elems, &mut |x| self.resolve(x))?; //elems.iter().map(|x| self.resolve(x)?).collect();
        // println!("{} ", builtins::_type(&vec![vals[0].clone()]) == builtins::_type(&vec![vals[1].clone()]));
        print_verbose!("compare {} {:?}", op, vals);
        match &op[..] {
            "==" => Ok(Block::Bool(vals[0]==vals[1])),
            "!=" => Ok(Block::Bool(vals[0]!=vals[1])),
            ">" => Ok(Block::Bool(vals[0]>vals[1])),
            "<" => Ok(Block::Bool(vals[0]<vals[1])),
            ">=" => Ok(Block::Bool(vals[0]>=vals[1])),
            "<=" => Ok(Block::Bool(vals[0]<=vals[1])),
            _ => eval_error!("Unsupported operator")
        }
    }

    pub fn eval_func_obj(&mut self, func: &Block, args: &Block, name: Option<&String>) -> Result<Block, KrustyErrorType> {
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
                let req_args = f.args.get_list().unwrap_or(vec![]); //.expect("function definition error");
                if req_args.len() != args.len() {
                    eval_error!(format!("function arguments for '{}' don't match", name));
                } else {
                    let mut exec_env = NameSpace::new(None, Some(self));
                    for (k,v) in req_args.iter().zip(args.iter()) {
                        exec_env.assign(&k, &v)?;
                    }
                    print_verbose!("CALL {} {:?}", name, f.body);
                    match &f.body { // return function result
                        Block::FuncBody(elist) => exec_env.run(&elist),
                        _ => eval_error!(format!("function '{}' definition error", name)),
                    }
                }
            },
            Block::NativeFunc(f) => {
                let clean_args: Vec<Block> = NameSpace::resolve_vector(&args, &mut |x| self.resolve(x))?;
                (f.func)(self, &clean_args)
            }
            _ => eval_error!(format!("function '{}' definition error", name))
        }
    }

    fn eval_func(&mut self, name: &String, args: &Block) -> Result<Block, KrustyErrorType> {
        // println!("<F> {:?}", args);
        match self.get(name) {
            Err(_) => eval_error!(format!("function '{}' not defined", name)),
            Ok(func) => {
                self.eval_func_obj(&func, args, Some(name))
            },
        }
    }

    fn pick_index(&self, idx: &Block, things: &Block) -> Result<Block, KrustyErrorType> {
        // println!("{:?} [{:?}]", things, idx);
        match (idx, things) {
            (Block::Object(Token::Number(n)), Block::List(a)) => Ok(a[*n as usize].clone()),
            (Block::Object(Token::Number(n)), Block::Object(Token::Text(a))) => {
                Ok(Block::Object(Token::Text(a.chars().nth(*n as usize).unwrap().to_string())))
            },
            _ => eval_error!(format!("cannot index {:?} with {:?}", things, idx))
        }
        // Block::Null
    }

    fn solve_expr(&mut self, exp: &Expression) -> Result<Block, KrustyErrorType> {
        // println!("<E> {:?}", exp);
        match &exp.op {
            Block::Operator(Token::Assign) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    eval_error!("Illegal assignment");
                }
                self.assign(&exp.elems[0], &exp.elems[1])?;
                Ok(Block::Null)
            },
            Block::Operator(Token::Arith(op)) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    eval_error!("Illegal arithmetic operation");
                }
                self.solve_arith(*op, &exp.elems) // return
            },
            Block::Operator(Token::Comparison(op)) => {
                // elems should have only 2 members
                if exp.elems.len() != 2 {
                    eval_error!("Illegal comparison operation");
                }
                self.solve_comparison(op, &exp.elems) // return
            },
            Block::Operator(Token::FuncCall) => {
                match &exp.elems[0] {
                    Block::Object(Token::Symbol(func_name)) => self.eval_func(func_name, &exp.elems[1]),
                    Block::Func(_) => self.eval_func_obj(&exp.elems[0], &exp.elems[1], None),
                    Block::Expr(ex) => {
                        let func = self.solve_expr(&ex)?;
                        self.eval_func_obj(&func, &exp.elems[1], None)
                    },
                    _ => Ok(Block::Null),
                }
            },
            Block::Operator(Token::FuncReturn) => { // will only return a list type object??
                let ret_list: Vec<Block> = NameSpace::resolve_vector(&exp.elems, &mut |e| self.resolve(e))?;
                Ok(match ret_list.len() {
                    // We also unwrap these late evaluated lists in case it has 0 or 1 elements
                    0 => Block::Null,
                    1 => ret_list[0].clone(),
                    _ => Block::List(ret_list)
                })
            },
            Block::Operator(Token::List) => {
                // These are some List type Expressions still unconverted to Block::List
                // They are usually deep inside a function definition needing late evaluation
                let ret_list: Vec<Block> = NameSpace::resolve_vector(&exp.elems, &mut |e| self.resolve(e))?;
                Ok(match ret_list.len() {
                    // We also unwrap these late evaluated lists in case it has 0 or 1 elements
                    0 => Block::Null,
                    1 => ret_list[0].clone(),
                    _ => Block::List(ret_list)
                })
            },
            Block::Operator(Token::Index) => {
                if exp.elems.len() != 2 {
                    eval_error!("Illegal index operation");
                }
                let val = self.resolve(&exp.elems[0])?;
                let idx = self.resolve(&exp.elems[1])?;
                self.pick_index(&idx, &val)
            },
            Block::Operator(Token::Accessor) => {
                if exp.elems.len() != 2 {
                    eval_error!("Illegal access operation");
                }
                match self.resolve(&exp.elems[0])? {
                    Block::Mod(m) => {
                        match &exp.elems[1] {
                            Block::Object(Token::Symbol(s)) => {
                                let var = m.vars.get(s);
                                if var.is_some() {Ok(var.unwrap().clone())} else {eval_error!("member not found")}
                            },
                            Block::Expr(x) => {
                                match x.op {
                                    Block::Operator(Token::Assign) => eval_error!("cannot assign into module"),
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
                            _ => eval_error!("invalid use of '.' accessor")
                        }
                    },
                    _ => eval_error!("invalid use of '.' accessor")
                }
            }
            _ => exp.clone().to_block()
        }
    }
}
