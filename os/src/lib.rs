#[macro_use] extern crate krusty_core;
use krusty_core::lib::{moddef, helper};

pub mod os;
pub mod file_io;


#[no_mangle]
pub fn load_all(m_vars: &mut moddef::ModuleVars) {
    helper::load_func(m_vars, "listdir", os::_listdir);
    helper::load_func(m_vars, "getcwd", os::_getcwd);
    helper::load_func(m_vars, "remove", os::_remove);
	// file io
	helper::load_func(m_vars, "open", file_io::_fileopen);
	helper::load_func(m_vars, "create", file_io::_filecreate);
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
