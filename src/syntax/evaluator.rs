use std::collections::HashMap;

use crate::syntax::parser;


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
    vars: HashMap<String, parser::Obj>,
    parent: Option<&'a Env<'a>>,
}

impl<'a> Env<'a> {
    pub fn new(parent: Option<&'a Env<'a>>) -> Env<'a> {
        Env {
            vars: HashMap::new(),
            parent
        }
    }

    pub fn disperse(&mut self, elist: &'a parser::ExprList) {
        for o in &elist.exprs {
            self.solve_expr(o);
            println!("{:?}=>>", self);
        }
        // let vo: Vec<parser::Obj> = Vec::new();
    }

    fn get(&self, key: &String) -> Option<parser::Obj> {
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

    fn assign(&mut self, key: &'a parser::Obj, value: &'a parser::Obj) {
        if let parser::Obj::Symbol(var) = key {
            println!("{:?}", var);
            match value {
                parser::Obj::Num(_) | parser::Obj::Text(_) => {
                    self.vars.insert(var.to_string(), value.clone());
                },
                parser::Obj::Symbol(s) => {
                    let mo = self.get(s).unwrap();
                    self.vars.insert(var.to_string(), mo);
                },
                parser::Obj::Expr(x) => {
                    match self.solve_expr(x) {
                        Some(o) => {self.vars.insert(var.to_string(), o.clone());},
                        None => {self.vars.insert(var.to_string(), parser::Obj::Null);}
                    }
                },
                parser::Obj::Grouped(gx) => {
                    if gx.exprs.len()==1 {
                        match self.solve_expr(&gx.exprs[0]) {
                            Some(o) => {self.vars.insert(var.to_string(), o.clone());},
                            None => {self.vars.insert(var.to_string(), parser::Obj::Null);}
                        }
                    } else {
                        self.vars.insert(var.to_string(), parser::Obj::Null);
                    }
                    // match self.solve_expr(x) {
                    //     Some(o) => {self.vars.insert(var.to_string(), o.clone());},
                    //     None => {self.vars.insert(var.to_string(), parser::Obj::Null);}
                    // }
                },
                _ => {self.vars.insert(var.to_string(), parser::Obj::Null);},
            };
            //
        } else {
            panic!("LHS is not a valid symbol");
        }
    }


    fn solve_arith(&mut self, op: char, elems: &'a Vec<parser::Obj>) ->Result<parser::Obj, &str> {
        // elems should have only 2 members
        if elems.len() > 2 {
            return Err("Illegal arithmetic operation");
        }
        let mut res: f64 = if "+-".contains(op) {0.0} else {1.0};
        for e in elems.iter() {
            println!("{:?} {:?}", op, e);
            match e {
                parser::Obj::Num(n) => {
                    res = _arith_operate(res, *n, op);
                },
                parser::Obj::Expr(x) => {
                    match self.solve_expr(x) {
                        Some(o) => {
                            if let parser::Obj::Num(n) = o {
                                res = _arith_operate(res, n, op);
                            }
                        },
                        None => ()
                    };
                }
                _ => ()
            }
        }
        return Ok(parser::Obj::Num(res)) //return
    }


    fn solve_expr(&mut self, exp: &'a parser::Expression) -> Option<parser::Obj> {
        println!("{:?}", exp);
        match exp.op {
            parser::Operator::Arith(op) => {
                match self.solve_arith(op, &exp.elems) {
                    Ok(res) => Some(res),
                    Err(e) => panic!("{}", e)
                } // return
            },
            parser::Operator::Assign => {
                // elems should have only 2 members
                if exp.elems.len() > 2 {
                    panic!("Illegal assignment");
                }
                self.assign(&exp.elems[0], &exp.elems[1]);
                Some(parser::Obj::Null)
            },
            _ => None
        }
    }
}
