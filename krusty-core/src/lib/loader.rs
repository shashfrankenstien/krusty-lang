use crate::lib::funcdef;
use crate::syntax::parser::Obj;

use std::collections::HashMap;

pub fn load_func(hm: &mut HashMap<String, Obj>, name: &str, f: funcdef::NativeFuncType) {
    hm.insert(name.to_string(), Obj::NativeFunc(
        funcdef::NativeFuncDef::new(f, name)
    ));
}
