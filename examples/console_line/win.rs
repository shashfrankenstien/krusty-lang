use std::io::{self, Write};

// use std::thread;
// use std::time::Duration;



pub struct Editor {
    line: String,
}

impl Editor {
    pub fn new() -> Editor {
		ctrlc::set_handler(move || {
			print!("CTRL+C\n");
        	io::stdout().flush().unwrap();
		}).expect("Error setting Ctrl-C handler");

        Editor {
            line: String::new(),
        }
    }


    pub fn readline(&mut self, prompt: &String) -> &String {
		self.line.clear();
        print!("{}", prompt);
        // self.offset = prompt.len();
        // term.write_line("Hello World!").unwrap();
        // thread::sleep(Duration::from_millis(2000));
        // term.clear_line().unwrap();
        io::stdout().flush().unwrap();
		let stdin = io::stdin();
		// let mut handle = stdin.lock();
		let _chars = stdin.read_line(&mut self.line);


		&self.line
    }
}
