use std::collections::HashMap;
use std::cmp::Ordering;
use std::path::PathBuf;
use std::fs;
use libloading;

use crate::syntax::evaluator::NameSpace;
use crate::syntax::parser::Obj;
use crate::lib::loader;

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



#[derive(Debug)]
pub struct Module {
    pub vars: HashMap<String, Obj>,
    pub path: Option<PathBuf>,
    dylib: Option<libloading::Library>
}

impl Clone for Module {
    fn clone(&self) -> Module {
        match self.dylib {
            None => Module{
                vars: self.vars.clone(),
                path: self.path.clone(),
                dylib: None
            },
            Some(_) => {
                Module {
                    vars: self.vars.clone(),
                    path: self.path.clone(),
                    dylib: Some(libloading::Library::new(self.path.clone().unwrap()).expect("library load error"))
                }
            },
        }
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for Module {}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.vars.len().partial_cmp(&other.vars.len())
    }
}

impl Module {
    pub fn new(path: Option<&PathBuf>) -> Module {
        let path = match path {
            None => None,
            Some(p) => {
                let srcfile = fs::canonicalize(p).expect("No such File!");
                Some(srcfile)
            }
        };
        Module {
            vars: HashMap::new(),
            path,
            dylib: None
        }
    }

    pub fn load_dylib(&mut self) {
        let path = self.path.clone().unwrap();
        let lib = libloading::Library::new(path).expect("library load error");
        unsafe {
            let func: libloading::Symbol<loader::DynLoadSignature> = lib.get(b"load").expect("library load error2");
            func(&mut self.vars);
        }
        self.dylib = Some(lib);
    }
}
