use std::cmp::Ordering;
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;

use libloading;
use lazy_static::lazy_static; // 1.4.0
use std::sync::Mutex;

use crate::syntax::parser::Block;


lazy_static! {
    // these refs are retained statically since extern function pointer copies are referencing these memory locations
    static ref _DYLIB_REFS: Mutex<HashMap<PathBuf, libloading::Library>> = Mutex::new(HashMap::new());
}

pub type ModuleVars = HashMap<String, Block>;
pub type DynLoadSignature = fn(&mut ModuleVars);


#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub vars: ModuleVars,
    pub path: Option<PathBuf>,
}


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
        }
    }

    fn _load_dylib_funcs(&mut self, lib: &libloading::Library) {
        unsafe {
            let load_func: libloading::Symbol<DynLoadSignature> = lib.get(b"load").expect("library load error2");
            load_func(&mut self.vars);
        }
    }

    pub fn load_dylib(&mut self) {
        let path = self.path.clone().unwrap();
        let loaded = match _DYLIB_REFS.lock().unwrap().get(&path) {
            Some(l) => {
                self._load_dylib_funcs(l);
                true
            }
            None => false,
        }; // release lock

        if !loaded {
            let l = libloading::Library::new(&path).expect("library load error");
            self._load_dylib_funcs(&l);
            _DYLIB_REFS.lock().unwrap().insert(path.clone(), l);
        }
    }
}
