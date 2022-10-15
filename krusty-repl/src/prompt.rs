use std::ops::Drop;
use std::path::PathBuf;

use rustyline::{self, error::ReadlineError, config::Configurer};


#[derive(Debug)]
pub struct Prompt {
    cli: rustyline::Editor::<()>,
    want_pair: Vec<char>,
    line_count: i32,
    hist_path: PathBuf,
    hist_len: usize
}

impl Prompt {
    pub fn new(hist_path: PathBuf, hist_len: usize) -> Result<Prompt, ReadlineError> {
        let mut cli = rustyline::Editor::<()>::new()?;
        cli.set_max_history_size(hist_len);
        cli.load_history(&hist_path).unwrap_or(());
        Ok(Prompt{
            cli,
            want_pair: Vec::new(),
            line_count: 0,
            hist_path,
            hist_len
        })
    }

    fn _match_pair(c: char) -> Option<char> {
        match c {
            '{'=> Some('}'),
            '['=> Some(']'),
            '('=> Some(')'),
            '"' => Some('"'),
            '\'' => Some('\''),
            _ => None
        }
    }

    fn is_complete(&mut self, s: &str) -> bool {
        for c in s.chars() {
            if self.want_pair.len() > 0 && c == self.want_pair[self.want_pair.len()-1] {
                self.want_pair.pop();
            }
            else if let Some(want) = Prompt::_match_pair(c) {
                self.want_pair.push(want);
            }
            else if self.want_pair.len() == 0 && c==';' {
                return true; // want nothing, eol found
            }
        };
        false
    }

    pub fn read_expr(&mut self) -> Result<String, ReadlineError> {
        self.want_pair.clear();
        let mut tot_chars = 0;

        #[cfg(windows)]
        let mut buffer = self.cli.readline(">> ")?; // color prompt on windows has length mesurement issues
        #[cfg(not(windows))]
        let mut buffer = self.cli.readline(&BLUE!(">> "))?;

        let mut chars = buffer.len();
        while buffer.trim()!="" {
            buffer.push('\n'); // rustyline removes newline character. Adding one back here
            if self.is_complete(&buffer[tot_chars..]) {break;}
            // print_verbose!("want_pair: {:?}", self.want_pair);

            #[cfg(windows)]
            let more = self.cli.readline(".. ")?; // color prompt on windows has length mesurement issues
            #[cfg(not(windows))]
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
        self.cli.save_history(&self.hist_path).unwrap();
    }
}


