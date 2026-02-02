mod error;
mod lock;
mod package;
mod verb;
mod yarn;

use std::env;

use crate::verb::Verb;
use crate::yarn::Yarn;

pub use error::Error;

const VERSION: &str = "2.0.0";

fn main() -> Result<(), Error> {
    let verb = Verb::try_from(env::args())?;

    match verb {
        Verb::Auto(no_install) => {
            let mut yarn = Yarn::new()?;
            yarn.print_info();
            if !no_install {
                yarn.install()?;
                yarn.dedupe()?;
                yarn.audit()?;
            } else {
                yarn.resolve("react@18.3.1", "^19")?;
            }
        }

        Verb::Reset(packages, no_install) => {
            let mut yarn = Yarn::new()?;
            let dirty = yarn.reset(&packages)?;
            if dirty && !no_install {
                yarn.install()?;
                yarn.dedupe()?;
                yarn.audit()?;
            }
        }

        Verb::Help => {
            println!("gnarl {} - the yarn v4 companion tool", VERSION);
            println!("usage: gnarl [<auto | reset | info | help> <args>]");
            println!("> gnarl [auto] [-n]");
            println!("> gnarl reset [-n] package-names...");
            println!("> gnarl info");
            println!("> gnarl help");
        }

        Verb::Info => {
            let yarn = Yarn::new()?;
            yarn.print_info();
        }
    };

    Ok(())
}
