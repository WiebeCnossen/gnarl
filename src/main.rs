use std::env;

use gnarl::cmd::{Command, Verb::*};
use gnarl::gnarl::Gnarl;
use gnarl::{Error, out_indent, out_info};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Error> {
    let command = Command::try_from(env::args())?;

    out_info!("gnarl {VERSION}");
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
            out_info!("the yarn v4 companion tool");
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
