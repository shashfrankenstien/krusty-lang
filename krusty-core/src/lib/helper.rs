/** this file contains helper functions used while defining an external dylib module
* external modules should expose a function `load_all` with signature defined by `DynLoadSignature`
* the `load_all` function can internally use `helper::load_func` to provide function pointers
*
* [#no_mangle]
* pub fn load_all(&mut helper::ModuleVars);
*
*/

use crate::lib::{funcdef, moddef};
use crate::syntax::parser::Block;


#[macro_export]
macro_rules! func_nargs_eq {
    ($vector:expr, $count:expr) => {
        if $vector.len() != $count {
            eval_error!(format!("expected {}, but received {} args", $count, $vector.len()))
        }
    };
}

#[macro_export]
macro_rules! func_nargs_le {
    ($vector:expr, $count:expr) => {
        if $vector.len() > $count {
            eval_error!(format!("expected 0..{}, but received {} args", $count, $vector.len()))
        }
    };
}

pub fn load_func(hm: &mut moddef::ModuleVars, name: &str, f: funcdef::NativeFuncType) {
    hm.insert(name.to_string(), Block::NativeFunc(
        funcdef::NativeFuncDef::new(f, name)
    ));
}
