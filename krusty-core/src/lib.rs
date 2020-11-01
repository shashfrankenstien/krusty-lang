#[macro_use]
pub mod macros;

pub mod syntax {
    pub mod lexer;
    pub mod parser;
    pub mod evaluator;
}

pub mod lib {
    pub mod builtins;
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}