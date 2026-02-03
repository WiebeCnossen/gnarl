mod audit;
mod cmd;
mod error;
mod gnarl;
mod lock;
mod npm;
mod package;
mod parse;
mod project;
mod yarn;

use std::env;

use crate::cmd::{Command, Verb::*};
use crate::gnarl::Gnarl;
use crate::yarn::Yarn;

pub use error::Error;
pub use package::Package;

const VERSION: &str = "2.0.0";

fn main() -> Result<(), Error> {
    let command = Command::try_from(env::args())?;

    match command.verb() {
        Auto => {
            let mut gnarl = Gnarl::new(command.options())?;
            gnarl.auto()?;
        }

        Reset => {
            let mut gnarl = Gnarl::new(command.options())?;
            gnarl.reset(command.parameters())?;
        }

        Check => {
            let mut gnarl = Gnarl::new(command.options())?;
            gnarl.check()?;
        }

        Help => {
            println!("gnarl {} - the yarn v4 companion tool", VERSION);
            println!("usage: gnarl [<auto | reset | check | info | help> <args>]");
            println!("> gnarl [auto] [-n]");
            println!("> gnarl reset [-n] package-names...");
            println!("> gnarl check");
            println!("> gnarl info");
            println!("> gnarl help");
        }

        Info => {
            let yarn = Yarn::new()?;
            yarn.print_info();
        }
    };

    Ok(())
}
