// pub enum Colors {
//     RED = 31,
//     GREEN = 32,
//     YELLOW = 93, //33
//     BLUE = 34,
//     PINK = 35,
//     LIGHTBLUE = 36,
//     WHITE = 37,
//     GRAY = 90,
//     CYAN = 96,
// }

// trait StringColorExt {
//     fn colorize(&self) -> String;
// }

// impl StringColorExt for String {
//     fn colorize(&self) -> String {

//     }
// }
#[macro_export]
#[allow(unused_macros)]
macro_rules! BLUE {
    ($plain_string:expr) => {format!("\x1B[1;36m{}\x1B[0m", $plain_string)};
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! GREEN {
    ($plain_string:expr) => {format!("\x1B[1;32m{}\x1B[0m", $plain_string)};
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! RED {
    ($plain_string:expr) => {format!("\x1B[1;31m{}\x1B[0m", $plain_string)};
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! YELLOW {
    ($plain_string:expr) => {format!("\x1B[1;93m{}\x1B[0m", $plain_string)};
}
