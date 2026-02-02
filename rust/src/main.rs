mod cmd;
mod error;
mod lock;
mod npm;
mod package;
mod yarn;

use std::env;

use crate::cmd::{Command, Verb::*};
use crate::npm::Npm;
use crate::yarn::Yarn;

pub use error::Error;

const VERSION: &str = "2.0.0";

fn main() -> Result<(), Error> {
    let command = Command::try_from(env::args())?;

    match command.verb() {
        Auto => {
            let mut yarn = Yarn::new()?;
            yarn.print_info();
            if !command.options().no_install() {
                yarn.install()?;
                yarn.dedupe()?;
                yarn.audit()?;
            } else {
                yarn.resolve("react@18.3.1", "^19")?;
            }
        }

        Reset => {
            let mut yarn = Yarn::new()?;
            let dirty = yarn.reset(command.parameters())?;
            if dirty && !command.options().no_install() {
                yarn.install()?;
                yarn.dedupe()?;
                yarn.audit()?;
            }
        }

        Help => {
            println!("gnarl {} - the yarn v4 companion tool", VERSION);
            println!("usage: gnarl [<auto | reset | info | help> <args>]");
            println!("> gnarl [auto] [-n]");
            println!("> gnarl reset [-n] package-names...");
            println!("> gnarl info");
            println!("> gnarl help");
        }

        Info => {
            let yarn = Yarn::new()?;
            yarn.print_info();

            let mut npm = Npm::new()?;
            for parameter in command.parameters() {
                npm.retrieve_packument(parameter)?;

                println!(
                    "{}: {} versions",
                    parameter,
                    npm.packument(parameter)?.versions().count()
                );

                if let Some(version) = npm.packument(parameter)?.versions().last() {
                    println!("{}@{}", parameter, version);
                    for key in npm.packument(parameter)?.version(version)?.dependencies() {
                        let value = npm
                            .packument(parameter)?
                            .version(version)?
                            .dependency(key)?;
                        println!("  {}: {}", key, value);
                    }
                }

                println!(
                    "{}: {} all versions",
                    parameter,
                    npm.packument(parameter)?.all_versions().count()
                );

                if let Some(version) = npm.packument(parameter)?.all_versions().last() {
                    println!("{}@{}", parameter, version);
                    for key in npm.packument(parameter)?.version(version)?.dependencies() {
                        let value = npm
                            .packument(parameter)?
                            .version(version)?
                            .dependency(key)?;
                        println!("  {}: {}", key, value);
                    }
                }
            }
        }
    };

    Ok(())
}
