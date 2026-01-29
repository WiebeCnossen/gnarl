mod error;
mod lock;
mod package;
mod verb;
mod yarn;

use std::env;
use std::process::exit;

use crate::yarn::Yarn;

pub use error::Error;

const VERSION: &str = "2.0.0";

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
        "auto" => {
            let yarn = Yarn::new()?;
            yarn.print_info();
        }
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
            let mut yarn = Yarn::new()?;
            let dirty = yarn.reset(&args[2..].iter().map(|s| s.as_str()).collect::<Vec<&str>>())?;
            if dirty {
                yarn.install().unwrap();
                yarn.dedupe().unwrap();
                yarn.audit().unwrap();
            }
        }
        _ => {
            eprintln!("unreachable verb {}", verb);
            exit(1);
        }
    };

    Ok(())
}
