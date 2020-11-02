use crate::syntax::evaluator::NameSpace;
use crate::syntax::parser::Phrase;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FuncDef {
    pub args: Phrase,
    pub body: Phrase
}


pub type NativeFuncType = fn(&mut NameSpace, args: &Vec<Phrase>) -> Phrase;

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


