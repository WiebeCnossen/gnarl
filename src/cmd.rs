use std::env::Args;
use std::io::IsTerminal;

use crate::audit::Severity;
use crate::ui::{UiMode, select_ui_mode};

#[derive(Debug, Clone, Copy)]
pub struct Options {
    no_install: bool,
    severity: Severity,
    raw: bool,
}

impl Options {
    fn read(args: &mut Vec<String>) -> Result<Self, crate::Error> {
        let mut no_install = false;
        let mut severity = Severity::Info;
        let mut raw = false;
        for i in (0..args.len()).rev() {
            match args[i].as_str() {
                "-x" => {
                    no_install = true;
                    args.remove(i);
                }
                "--raw" => {
                    raw = true;
                    args.remove(i);
                }
                "-s" => {
                    severity = args[i + 1].parse()?;
                    args.remove(i);
                    args.remove(i);
                }
                _ => {}
            }
        }

        Ok(Self {
            no_install,
            severity,
            raw,
        })
    }

    pub fn no_install(&self) -> bool {
        self.no_install
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn raw(&self) -> bool {
        self.raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Auto,
    Reset,
    Check,
    Info,
    Help,
}

pub struct Command {
    verb: Verb,
    options: Options,
    parameters: Vec<String>,
}

impl Command {
    pub fn verb(&self) -> Verb {
        self.verb
    }

    pub fn options(&self) -> Options {
        self.options
    }

    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    pub fn ui_mode(&self) -> UiMode {
        select_ui_mode(std::io::stdout().is_terminal(), self.options.raw(), self.verb)
    }
}

impl TryFrom<Args> for Command {
    type Error = crate::Error;
    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let mut parameters = args.skip(1).collect::<Vec<String>>();
        let options = Options::read(&mut parameters)?;

        if parameters.is_empty() {
            return Ok(Self {
                verb: Verb::Auto,
                options,
                parameters,
            });
        }

        let verb = match parameters.remove(0).as_str() {
            "auto" => Verb::Auto,
            "reset" => Verb::Reset,
            "check" => Verb::Check,
            "info" => Verb::Info,
            "help" => Verb::Help,
            unknown => return Err(format!("Unknown verb {}", unknown).into()),
        };

        Ok(Self {
            verb,
            options,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Command {
        let mut parameters: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
        let options = Options::read(&mut parameters).unwrap();
        let verb = if parameters.is_empty() {
            Verb::Auto
        } else {
            match parameters.remove(0).as_str() {
                "auto" => Verb::Auto,
                "reset" => Verb::Reset,
                "check" => Verb::Check,
                "info" => Verb::Info,
                "help" => Verb::Help,
                other => panic!("unknown {other}"),
            }
        };
        Command {
            verb,
            options,
            parameters,
        }
    }

    #[test]
    fn parses_raw_with_other_flags() {
        let cmd = parse(&["check", "--raw", "-x", "-s", "high"]);
        assert_eq!(cmd.verb(), Verb::Check);
        assert!(cmd.options().raw());
        assert!(cmd.options().no_install());
        assert_eq!(cmd.options().severity(), Severity::High);
    }

    #[test]
    fn raw_defaults_false() {
        let cmd = parse(&["auto"]);
        assert!(!cmd.options().raw());
    }
}
