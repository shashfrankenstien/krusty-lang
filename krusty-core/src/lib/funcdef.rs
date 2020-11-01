use std::collections::HashMap;
use std::cmp::Ordering;
use std::path::PathBuf;

use crate::syntax::evaluator::NameSpace;
use crate::syntax::parser::Obj;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FuncDef {
    pub args: Obj,
    pub body: Obj
}


pub type NativeFuncType = fn(&mut NameSpace, args: &Vec<Obj>) -> Obj;

#[derive(Clone)]
pub struct NativeFuncDef {
    pub func: NativeFuncType,
    pub name: String
}

impl NativeFuncDef {
    pub fn new(func:NativeFuncType, name: &str) -> NativeFuncDef {
        NativeFuncDef {
            func,
            name:name.to_string()
        }
    }
}

impl std::fmt::Debug for NativeFuncDef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("NativeFuncDef")
            .field("name", &self.name)
            .finish()
    }
}

impl PartialEq for NativeFuncDef {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for NativeFuncDef {}


impl PartialOrd for NativeFuncDef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}




#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub vars: HashMap<String, Obj>,
    pub path: Option<PathBuf>
}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.vars.len().partial_cmp(&other.vars.len())
    }
}

impl Module {
    pub fn new(path: Option<PathBuf>) -> Module {
        Module {
            vars: HashMap::new(),
            path
        }
    }
}
