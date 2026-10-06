use std::env;
use std::io::IsTerminal;
use std::process::ExitCode;

use gnarl::cmd::{Command, Verb, Verb::*, help_lines};
use gnarl::gnarl::Gnarl;
use gnarl::ui::{UiMode, select_ui_mode, stdout_reporter, tui};
use gnarl::{Error, RunStatus, out_indent, out_info};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match run() {
        Ok(status) => ExitCode::from(status.policy_exit()),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<RunStatus, Error> {
    let command = Command::try_from(env::args())?;

    match command.verb() {
        Auto => run_verb(command.verb(), command.options()),

        Reset => {
            // Standalone reset (and any chained auto) starts on stdout for the reset lines.
            out_info!("gnarl {VERSION}");
            let mut gnarl = Gnarl::with_reporter(command.options(), stdout_reporter())?;
            let should_auto = gnarl.reset(command.parameters())?;
            if should_auto {
                // Version already printed above for the reset stdout path.
                run_verb_inner(Auto, command.options(), false, true)
            } else {
                Ok(RunStatus::ok())
            }
        }

        Check => run_verb(command.verb(), command.options()),

        Help => {
            out_info!("gnarl {VERSION}");
            for line in help_lines() {
                if line.starts_with("the yarn") {
                    out_info!("{line}");
                } else {
                    out_indent!("{line}");
                }
            }
            Ok(RunStatus::ok())
        }

        Info => {
            out_info!("gnarl {VERSION}");
            out_info!("");
            Ok(RunStatus::ok())
        }
    }
}

fn run_verb(verb: Verb, options: gnarl::cmd::Options) -> Result<RunStatus, Error> {
    run_verb_inner(verb, options, true, false)
}

fn run_verb_inner(
    verb: Verb,
    options: gnarl::cmd::Options,
    print_version: bool,
    refresh_first: bool,
) -> Result<RunStatus, Error> {
    let mode = select_ui_mode(std::io::stdout().is_terminal(), options.raw(), verb);
    match mode {
        UiMode::Interactive => tui::run_interactive(verb, options, VERSION, refresh_first),
        UiMode::Stdout => {
            if print_version {
                out_info!("gnarl {VERSION}");
            }
            let mut gnarl = Gnarl::with_reporter(options, stdout_reporter())?;
            match verb {
                Auto => gnarl.auto(refresh_first),
                Check => gnarl.check(),
                _ => Ok(RunStatus::ok()),
            }
        }
    }
}
