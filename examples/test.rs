use console::Term;

use std::io::{self, Write};

// use std::thread;
// use std::time::Duration;

// pub struct simpleline {
//     cli: console::Term,
// }
macro_rules! BLUE {
    ($plain_string:expr) => {format!("\x1B[1;36m{}\x1B[0m", $plain_string)};
}


fn main() {
    let term = Term::stdout();
    print!("{}", BLUE!("hello"));

    io::stdout().flush().unwrap();
    // term.write_line("Hello World!").unwrap();
    // thread::sleep(Duration::from_millis(2000));
    // term.clear_line().unwrap();
    loop {
        let k = term.read_key();

        print!("{:?}", k);
        io::stdout().flush().unwrap();
    }
}
