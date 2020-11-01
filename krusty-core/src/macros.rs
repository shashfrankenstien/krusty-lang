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

