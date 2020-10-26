use std::env;
use std::ops::Drop;

use rustyline::{self, error::ReadlineError, config::Configurer};

const HISTFILE: &'static str = "history.txt";
const HISTLEN: usize = 20;


#[derive(Debug)]
pub struct Prompt {
    cli: rustyline::Editor::<()>,
    want_quote: Option<char>,
    want_pair: Option<char>,
    line_count: i32,
}

impl Prompt {
    pub fn new() -> Prompt {
        let mut cli = rustyline::Editor::<()>::new();
        cli.set_max_history_size(HISTLEN);
        cli.load_history(HISTFILE).unwrap_or(());
        Prompt{
            cli,
            want_quote: None,
            want_pair: None,
            line_count: 0,
        }
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
                self.want_quote = if want==c { None } else { Some(want) };
            }
            else if let Some(want) = self.want_pair {
                // print_verbose!("Here! {} {}", want, c);
                self.want_pair = if want==c { None } else { Some(want) };
            }
            else if c == '"' || c == '\'' {
                self.want_quote = Some(c);
            } else if let Some(want) = Prompt::_match_pair(c) {
                self.want_pair = Some(want);
            }
            else if c==';' {
                return true; // want nothing, eol found
            }
        };
        false
    }

    pub fn read_expr(&mut self) -> Result<String, ReadlineError> {
        self.want_quote = None;
        self.want_pair = None;
        let mut tot_chars = 0;

        let mut buffer = self.cli.readline(&BLUE!(">> "))?;
        let mut chars = buffer.len();
        while buffer.trim()!="" {
            buffer.push('\n'); // rustyline removes newline character. Adding one back here
            if self.is_complete(&buffer[tot_chars..]) {break;}
            print_verbose!("want_quote: {:?}, want_pair: {:?}", self.want_quote, self.want_pair);
            let more = self.cli.readline(&BLUE!(".. "))?;
            buffer.push_str(&more);
            tot_chars += chars + 1; // +1 for the added newline
            chars = more.len();
        }
        // println!("{:?}", buffer);
        self.cli.add_history_entry(buffer.trim());
        Ok(buffer)
    }
}

impl Drop for Prompt {
    fn drop(&mut self) {
        self.cli.save_history(HISTFILE).unwrap();
    }
}


