mod audit;
mod check;
mod cmd;
mod error;
mod gnarl;
mod locks;
mod npm;
mod package;
mod parse;
mod project;
mod ux;
mod yarn;

use std::env;

use crate::cmd::{Command, Verb::*};
use crate::gnarl::Gnarl;

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
            out_info!("gnarl {} - the yarn v4 companion tool", VERSION);
            out_indent!("usage: gnarl [<auto | reset | check | info | help> <args>]");
            out_indent!("> gnarl [auto] [-n]");
            out_indent!("> gnarl reset [-n] package-names...");
            out_indent!("> gnarl check");
            out_indent!("> gnarl info");
            out_indent!("> gnarl help");
        }

        Info => {
            out_info!("");
        }
    };

    Ok(())
}
