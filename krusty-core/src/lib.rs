#[macro_use]
pub mod macros;



pub mod syntax {
    pub mod lexer;
    pub mod parser;
    pub mod evaluator;
    mod lexer_tweaks;
}


pub mod lib {
    pub mod funcdef;
    pub mod builtins;
    pub mod helper;
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
