/** this file contains helper functions used while defining an external dylib module
* external modules should expose a function `load` with signature defined by `DynLoadSignature`
* the `load` function can internally use `loader::load_func` to provide function pointers
*
* [#no_mangle]
* pub fn load(&mut loader::ModuleVars);
*
*/

use crate::lib::funcdef;
use crate::syntax::parser::Phrase;

use std::collections::HashMap;

pub type ModuleVars = HashMap<String, Phrase>;
pub type DynLoadSignature = fn(&mut ModuleVars);

pub fn load_func(hm: &mut ModuleVars, name: &str, f: funcdef::NativeFuncType) {
    hm.insert(name.to_string(), Phrase::NativeFunc(
        funcdef::NativeFuncDef::new(f, name)
    ));
}
