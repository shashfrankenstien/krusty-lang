#[macro_export]
macro_rules! print_verbose {
    ($plain_string:expr) => {
        if env::var("VERBOSE").is_ok() {
            println!($plain_string);
        }
    };
    ($fmt_string:expr, $( $args:expr ),*) => {
        if env::var("VERBOSE").is_ok() {
            println!($fmt_string, $($args),* );
        }
    };
}
