// #[cfg(not(windows))]
mod console_line {
    pub mod unix;
}

// #[cfg(not(windows))]
use console_line::unix::Editor;

// #[cfg(windows)]
// mod console_line {
//     pub mod win;
// }

// #[cfg(windows)]
// use console_line::win::Editor;


macro_rules! BLUE {
    ($plain_string:expr) => {format!("\x1B[1;36m{}\x1B[0m", $plain_string)};
}


fn main() {
    let mut term = Editor::new();

    loop {
        let s = term.readline(&BLUE!("kru> "));
        // let s = term.readline(&String::from("\x1B[6n"));
        print!("out-{}", s);
        if s.trim()=="exit" {
            break;
        }
    }
}
