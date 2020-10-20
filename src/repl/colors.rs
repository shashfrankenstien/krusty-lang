// RED = 31,
// GREEN = 32,
// YELLOW = 93, //33,
// BLUE = 34,
// PINK = 35,
// LIGHTBLUE = 36,
// WHITE = 37,
// GRAY = 90,
// CYAN = 96,

// impl fmt::Display for Colors {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

macro_rules! BLUE {
    ($plain_string:expr) => {format!("\x1B[1;36m{}\x1B[0m", $plain_string)};
}

// macro_rules! RED {
//     ($plain_string:expr) => {
//         format!("\033[{}m{}\033[0m", Colors::BLUE, $plain_string);
//     }
// }
// macro_rules! BLUE {
//     ($plain_string:expr) => {
//         format!("\033[{}m{}\033[0m", Colors::BLUE, $plain_string);
//     }
// }
// macro_rules! BLUE {
//     ($plain_string:expr) => {
//         format!("\033[{}m{}\033[0m", Colors::BLUE, $plain_string);
//     }
// }
// macro_rules! BLUE {
//     ($plain_string:expr) => {
//         format!("\033[{}m{}\033[0m", Colors::BLUE, $plain_string);
//     }
// }
