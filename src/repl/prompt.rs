use std::env;

use rustyline::{self, error::ReadlineError};

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

    fn is_complete(&mut self, s: &str) -> bool {
        for c in s.chars() {
            if let Some(want) = self.want_quote {
                // print_verbose!("Here! {} {}", want, c);
                self.want_quote = if want==c {
                    None
                 } else { Some(want) };
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


pub fn prompt(rl: &mut rustyline::Editor::<()>, _line: i32) -> Result<String, ReadlineError> {

    let mut expr_tracker = ExprTracker::new();
    let mut tot_chars = 0;

    let mut buffer = rl.readline(&BLUE!(">> "))?;
    let mut chars = buffer.len();
    while buffer.trim()!="" {
        if expr_tracker.is_complete(&buffer[tot_chars..]) {break;}
        print_verbose!("{:?}", expr_tracker);
        let more = rl.readline(&BLUE!(".."))?;
        buffer.push_str(&more);
        tot_chars += chars;
        chars = more.len();
    }
    // println!("{:?}", buffer);
    rl.add_history_entry(buffer.as_str());
    Ok(buffer)
}
