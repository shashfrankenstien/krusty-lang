use std::io::{self, BufRead, Write};


#[derive(Debug)]
struct ExprTracker {
    want_quote: Option<char>,
    want_pair: Option<char>,
}

impl ExprTracker {
    fn new() -> ExprTracker {
        ExprTracker{want_quote:None, want_pair:None}
    }

    fn _match_pair(c: char) -> Option<char> {
        match c {
            '{'=> Some('}'),
            '['=> Some(']'),
            '('=> Some(')'),
            _ => None
        }
    }

    fn is_complete(&mut self, s: &String) -> bool {
        for c in s.chars() {
            if let Some(want) = self.want_quote {
                self.want_quote = if want==c { None } else { Some(want) };
            }
            else if let Some(want) = self.want_pair {
                self.want_pair = if want==c { None } else { Some(want) };
            }
            else if c == '"' || c == '\'' {
                self.want_quote = Some(c);
            } else if let Some(want) = ExprTracker::_match_pair(c) {
                self.want_pair = Some(want);
            }
            else if c==';' {
                return true; // want nothing, eol found
            }
        };
        false
    }
}


pub fn prompt(_line: i32) -> Option<String> {
    // print!("[{}]", BLUE!(_line));
    print!("{} ", BLUE!(">>"));
    // print!("\u{1F980}");

    io::stdout().flush().unwrap();
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut expr_tracker = ExprTracker::new();
    let mut chars = handle.read_line(&mut buffer);
    while chars.is_ok() && buffer.trim()!="" {
        if expr_tracker.is_complete(&buffer) {break;}
        print!("{} ", BLUE!(".."));
        io::stdout().flush().unwrap();
        chars = handle.read_line(&mut buffer);
    }
    // println!("{:?}", buffer);
    if chars.is_ok() {
        Some(buffer)
    } else {
        None
    }
}
