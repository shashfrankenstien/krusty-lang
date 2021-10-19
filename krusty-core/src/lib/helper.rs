/** this file contains helper functions used while defining an external dylib module
* external modules should expose a function `load_all` with signature defined by `DynLoadSignature`
* the `load_all` function can internally use `helper::load_func` to provide function pointers
*
* [#no_mangle]
* pub fn load_all(&mut helper::ModuleVars);
*
*/

use std::path::PathBuf;

use crate::lib::{funcdef, moddef};
use crate::syntax::parser::Block;


#[macro_export]
macro_rules! func_nargs_eq {
    ($vector:expr, $count:expr) => {
        if $vector.len() != $count {
            panic!("expected {}, but received {} args", $count, $vector.len())
        }
    };
}

#[macro_export]
macro_rules! func_nargs_le {
    ($vector:expr, $count:expr) => {
        if $vector.len() > $count {
            panic!("expected 0..{}, but received {} args", $count, $vector.len())
        }
    };
}

pub fn load_func(hm: &mut moddef::ModuleVars, name: &str, f: funcdef::NativeFuncType) {
    hm.insert(name.to_string(), Block::NativeFunc(
        funcdef::NativeFuncDef::new(f, name)
    ));
}


pub fn convert_dylib_os_name(p: &mut PathBuf) {
    let mut fname = p.file_name()
        .expect("filename not valid")
        .to_str()
        .expect("filename not valid")
        .to_owned();

    #[cfg(target_os = "windows")]
    {
        if !fname.ends_with(".dll") {
            fname = fname + ".dll"
        }
        p.set_file_name(fname);
    }
    #[cfg(target_os = "macos")]
    {
        if !fname.starts_with("lib") {
            fname = "lib".to_owned() + &fname;
        }
        if !fname.ends_with(".dylib") {
            fname = fname + ".dylib"
        }
        p.set_file_name(fname);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if !fname.starts_with("lib") {
            fname = "lib".to_owned() + &fname;
        }
        if !fname.ends_with(".so") {
            fname = fname + ".so"
        }
        p.set_file_name(fname);
    }
}
