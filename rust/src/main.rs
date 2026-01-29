mod lock;
mod package;
mod yarn;

use std::env;
use std::process::exit;

use crate::yarn::{Yarn};

const VERSION: &str = "2.0.0";

#[derive(Debug)]
enum Error {
    Str(&'static str),
    String(String),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Str(s)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Str(s) => write!(f, "{}", s),
            Error::String(s) => write!(f, "{}", s),
        }
    }
}

fn help() {
    println!("gnarl {} - the yarn v4 companion tool", VERSION);
    println!("usage: gnarl [<auto | audit | check | fix | help | reset> <args>]");
    println!("> gnarl [auto]");
    println!("> gnarl audit");
    println!("> gnarl check");
    println!("> gnarl fix package-name safe-version-request");
    println!("> gnarl help");
    println!("> gnarl reset package-names...");
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    let verb = if args.len() > 1 {
        match args[1].as_str() {
            "audit" | "check" | "fix" | "help" | "reset" => args[1].as_str(),
            _ => {
                eprintln!("unknown verb: {}", args[1]);
                exit(1);
            }
        }
    } else {
        "auto"
    };

    match verb {
        "auto" => loop {
            let yarn = Yarn::new()?;
            yarn.print_info();
            yarn.install().unwrap();
            yarn.dedupe().unwrap();
            yarn.audit().unwrap();
            break;
        },
        "audit" => {
            println!("gnarl audit");
        }
        "check" => {
            println!("gnarl check");
        }
        "fix" => {
            println!("gnarl fix");
        }
        "help" => {
            help();
        }
        "reset" => {
            println!("gnarl reset");
        }
        _ => {
            eprintln!("unreachable verb {}", verb);
            exit(1);
        }
    };

    Ok(())
}
