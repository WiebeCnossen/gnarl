use std::env;
use std::io::IsTerminal;

use gnarl::cmd::{Command, Verb, Verb::*};
use gnarl::gnarl::Gnarl;
use gnarl::ui::{UiMode, select_ui_mode, stdout_reporter, tui};
use gnarl::{Error, out_indent, out_info};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Error> {
    let command = Command::try_from(env::args())?;

    match command.verb() {
        Auto => run_verb(command.verb(), command.options())?,

        Reset => {
            // Standalone reset (and any chained auto) starts on stdout for the reset lines.
            out_info!("gnarl {VERSION}");
            let mut gnarl = Gnarl::with_reporter(command.options(), stdout_reporter())?;
            let should_auto = gnarl.reset(command.parameters())?;
            if should_auto {
                // Version already printed above for the reset stdout path.
                run_verb_inner(Auto, command.options(), false)?;
            }
        }

        Check => run_verb(command.verb(), command.options())?,

        Help => {
            out_info!("gnarl {VERSION}");
            out_info!("the yarn v4 companion tool");
            out_indent!("usage: gnarl [<auto | reset | check | info | help> <args>]");
            out_indent!("> gnarl [auto] [--raw] [-x] [-s <severity>]");
            out_indent!("> gnarl reset [--raw] [-x] package-names...");
            out_indent!("> gnarl check [--raw]");
            out_indent!("> gnarl info");
            out_indent!("> gnarl help");
            out_indent!("--raw  force tagged stdout even on a TTY");
        }

        Info => {
            out_info!("gnarl {VERSION}");
            out_info!("");
        }
    };

    Ok(())
}

fn run_verb(verb: Verb, options: gnarl::cmd::Options) -> Result<(), Error> {
    run_verb_inner(verb, options, true)
}

fn run_verb_inner(verb: Verb, options: gnarl::cmd::Options, print_version: bool) -> Result<(), Error> {
    let mode = select_ui_mode(std::io::stdout().is_terminal(), options.raw(), verb);
    match mode {
        UiMode::Interactive => tui::run_interactive(verb, options, VERSION),
        UiMode::Stdout => {
            if print_version {
                out_info!("gnarl {VERSION}");
            }
            let mut gnarl = Gnarl::with_reporter(options, stdout_reporter())?;
            match verb {
                Auto => gnarl.auto(),
                Check => gnarl.check(),
                _ => Ok(()),
            }
        }
    }
}
