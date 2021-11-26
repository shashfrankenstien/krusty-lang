use std::path::PathBuf;
use path_slash::PathBufExt; // for PatjBuf::from_slash() trait
use std::env; // required for print_verbose! macro

use clap::Parser;


#[macro_use] extern crate krusty_repl;
use krusty_repl::prompt;

#[macro_use] extern crate krusty_core;
use krusty_core::syntax::lexer;
use krusty_core::syntax::parser;
use krusty_core::syntax::evaluator;
use krusty_core::lib::errors::{Error, KrustyErrorType};

use krusty_core::lib::pkg;


pub const APP_NAME_STR: &'static str = env!("CARGO_PKG_NAME");
pub const VERSION_STR: &'static str = env!("CARGO_PKG_VERSION");




fn is_sysexit(err: &KrustyErrorType) -> bool {
    match err.as_any().downcast_ref::<Error>() {
        Some(Error::SysExit{..}) => true,
        _ => false
    }
}


fn repl_run_line(ns: &mut evaluator::NameSpace, buf: &String) -> Result<parser::Block, KrustyErrorType> {
    let mut tokens = lexer::lex(&buf)?;
    let parsed = parser::parse(&mut tokens)?;
    let blk = ns.run(&parsed)?;
    match blk {
        parser::Block::Null => (),
        _ => println!("{}", blk)
    }
    Ok(blk)
}


fn repl_prompt() {
    println!(
        "{} {} {} {}",
        GREEN!("Welcome to Krusty"),
        GREEN!(VERSION_STR), "\u{1F980}",
        GREEN!("repl. Ctrl+C or exit() to quit!")
    );
    let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
    let mut ns = evaluator::NameSpace::new(Some(&cwd), None);

    let mut cli = prompt::Prompt::new();
    loop {
        let buffer = cli.read_expr();
        match buffer {
            Ok(buf) if buf.trim().len() == 0 => (),
            Ok(buf) => {
                match repl_run_line(&mut ns, &buf) {
                    Ok(_) => (),
                    Err(e) => {
                        if is_sysexit(&e) {
                            break;
                        } else {
                            println!("{}: {}", RED!("Error in expression"), buf.trim());
                            println!("{}", e.msg());
                        }
                    }
                }
            },
            Err(e) => { // ctrl-c or ctrl-d
                println!("{}", RED!(e.to_string()));
                break;
            }
        }
    }
}


fn run_file(filepath: &PathBuf) -> Result<(), KrustyErrorType> {
    let mut ns = evaluator::NameSpace::new(Some(filepath), None);
    print_verbose!("Running {:?}", ns.get_path());

    let mut tokens = lexer::lex_file(filepath)?;
    let tree = parser::parse(&mut tokens)?;
    let _vo = ns.run(&tree)?;

    print_verbose!("FINAL\n{:?}\n{:?}", _vo, ns);
    Ok(())
}


// fn install_native_module(path)




/// Experimental programing language written in Rust
#[derive(Parser)]
#[clap(name = APP_NAME_STR)]
pub struct CliOpts {

    /// script files to run (List)
    pub scripts: Vec<String>,

    /// package paths to install
    #[clap(short, long, name = "packages")]
    pub install: Option<Vec<String>>,
}



fn main() -> Result<(), i8> {
    let mut success: bool = true;
    let cli = CliOpts::parse();

    if cli.scripts.len() > 0 {
        for f in &cli.scripts {
            let filepath = PathBuf::from_slash(f);
            if filepath.is_file() {
                match run_file(&filepath) {
                    Ok(_) => (),
                    Err(e) => {
                        if !is_sysexit(&e) {
                            e.print_traceback();
                            success = false;
                        }
                        break;
                    }
                }
            }
        }
    }
    else if cli.install.is_some() {
        println!("{:?}", cli.install);
        for f in &cli.install.unwrap() {
            if let Err(e) = pkg::install_pkg(f) {
                println!("{:?}", e.msg());
                return Err(1)
            }
        }
    }
    else {
        repl_prompt();
    }

    if success {
        Ok(())
    } else {
        Err(1)
    }
}
