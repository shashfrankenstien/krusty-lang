use std::io::{self, BufRead, Write};

fn expr_is_complete(s: &String) -> bool {
    let mut need_close = false;
    let mut eol_found = false;
    for c in s.chars() {
        need_close = match (c, need_close) {
            ('"', _) => !need_close,
            ('(', false) => true, // stay open till ')'
            ('[', false) => true, // stay open till ']'
            ('{', false) => true, // stay open till '}'
            (')', true) => false,
            (']', true) => false,
            ('}', true) => false,
            (';', false) => {
                eol_found = true;
                need_close
            },
            (_,_) => need_close
        }
    };
    eol_found && !need_close // if need_close is true, expression is not complete
}

pub fn prompt(_line: i32) -> Option<String> {
    // print!("[{}]", BLUE!(_line));
    print!("{} ", BLUE!(">>"));
    // print!("\u{1F980}");

    io::stdout().flush().unwrap();
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut chars = handle.read_line(&mut buffer);
    while chars.is_ok() && !expr_is_complete(&buffer) && buffer.trim()!="" {
        chars = handle.read_line(&mut buffer);
    }
    // println!("{:?}", buffer);
    if chars.is_ok() {
        Some(buffer)
    } else {
        None
    }
}
