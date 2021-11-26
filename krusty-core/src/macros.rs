#[macro_export]
macro_rules! generic_error {
    ($plain_string:expr) => {
        return Err(Box::new(Error::GenericError{msg: $plain_string.to_string(), fname: String::from(""), lino: -1}))
    };
}


#[macro_export]
macro_rules! lex_error {
    ($plain_string:expr) => {
        return Err(Box::new(Error::LexerError{msg: $plain_string.to_string(), fname: String::from(""), lino: -1}))
    };
}


#[macro_export]
macro_rules! parser_error {
    ($plain_string:expr) => {
        return Err(Box::new(Error::ParserError{msg: $plain_string.to_string(), fname: String::from(""), lino: -1}))
    };
}


#[macro_export]
macro_rules! eval_error {
    ($plain_string:expr) => {
        return Err(Box::new(Error::EvalError{msg: $plain_string.to_string(), fname: String::from(""), lino: -1}))
    };
}

#[macro_export]
macro_rules! import_error {
    ($plain_string:expr) => {
        return Err(Box::new(Error::ImportError{msg: $plain_string.to_string(), fname: String::from(""), lino: -1}))
    };
}


#[macro_export]
macro_rules! sys_exit_error {
    () => {
        return Err(Box::new(Error::SysExit{msg: "Exit".to_string(), fname: String::from(""), lino: -1}))
    };
}


#[macro_export]
macro_rules! print_verbose {
    ($plain_string:expr) => {
        #[cfg(debug_assertions)]
        if env::var("KRUSTY_VERBOSE").is_ok() {
            println!($plain_string);
        }
    };
    ($fmt_string:expr, $( $args:expr ),*) => {
        #[cfg(debug_assertions)]
        if env::var("KRUSTY_VERBOSE").is_ok() {
            println!($fmt_string, $($args),* );
        }
    };
}

#[macro_export]
macro_rules! print_verbose_iter {
    ($iterable:expr) => {
        #[cfg(debug_assertions)]
        if env::var("KRUSTY_VERBOSE").is_ok() {
            for (i,o) in $iterable.iter().enumerate() {
                println!("{} {:?}", i, o)
            }
        }
    };
}

