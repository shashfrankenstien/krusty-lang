use console::{Term, Key};

use std::io::{self, Write};

// use std::thread;
// use std::time::Duration;



pub struct Editor {
    cli: console::Term,
    line: String,
    pos: usize,
}

impl Editor {
    pub fn new() -> Editor {
		ctrlc::set_handler(move || {
			println!("CTRL+C");
		}).expect("Error setting Ctrl-C handler");

        Editor {
            cli: Term::stdout(),
            line: String::new(),
            pos: 0,
        }
    }

    fn _print_remainder(&self, move_right: bool) {
        let rem = &self.line[self.pos..];
		print!("{}", rem);
        io::stdout().flush().unwrap();
		if !move_right {
			self.cli.move_cursor_left(rem.len()).unwrap();
		}
    }

    fn push(&mut self, c: char) {
		self.line.insert(self.pos, c);
        self._print_remainder(true);
        self.pos += 1;
    }

    fn left(&mut self, n: usize) {
        if self.pos >= n {
            self.cli.move_cursor_left(n).unwrap();
            self.pos -= n
        };
    }

    fn right(&mut self, n: usize) {
		if self.pos < self.line.len() {
			self.cli.move_cursor_right(n).unwrap();
			self.pos += n;
		}
    }

    fn delete(&mut self) {
        if self.pos < self.line.len() {
            self.right(1);
            self.backspace();
        }
    }

    fn backspace(&mut self) {
        if self.pos > 0 {
			// clears all chars to the right of cursor in unix
			self.cli.clear_chars(1).unwrap();
			self.pos -= 1;
			self.line.remove(self.pos);

			#[cfg(windows)]
			{
				// on windows console, clear_chars does not clear all chars to the right like in unix
				let rem = self.line.len()-self.pos+1;
				self.cli.move_cursor_right(rem).unwrap();
				self.cli.clear_chars(rem).unwrap();
			}
			self._print_remainder(false);
		}
    }

    fn home(&mut self) {
        self.left(self.pos);
    }

    fn end(&mut self) {
        self.right(self.line.len()-self.pos);
    }

    pub fn readline(&mut self, prompt: &String) -> &String {
		self.line.clear();
		self.pos = 0;
        print!("{}", prompt);
        // self.offset = console::measure_text_width(prompt);
        // term.write_line("Hello World!").unwrap();
        // thread::sleep(Duration::from_millis(2000));
        // term.clear_line().unwrap();
        io::stdout().flush().unwrap();

        loop {
            let key = self.cli.read_key();
            if key.is_ok() {
                match key.unwrap() {
                    Key::Char(c) => self.push(c),
                    Key::Tab => self.push('\t'),
                    Key::ArrowLeft => self.left(1),
                    Key::ArrowRight => self.right(1),
                    Key::Home => self.home(),
                    Key::End => self.end(),
                    Key::Backspace => self.backspace(),
                    Key::Del => self.delete(),
                    Key::Enter => {
                        self.line.push('\n');
                        break;
                    },
                    _ => ()
                }
            }
        }
        print!("{}", '\n');
        io::stdout().flush().unwrap();

        &self.line
    }
}
