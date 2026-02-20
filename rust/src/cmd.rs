use std::env::Args;

use crate::audit::Severity;

#[derive(Debug, Clone, Copy)]
pub struct Options {
    no_install: bool,
    severity: Severity,
}

impl Options {
    fn read(args: &mut Vec<String>) -> Result<Self, crate::Error> {
        let mut no_install = false;
        let mut severity = Severity::Info;
        for i in (0..args.len()).rev() {
            match args[i].as_str() {
                "-x" => {
                    no_install = true;
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
        })
    }

    pub fn no_install(&self) -> bool {
        self.no_install
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }
}

#[derive(Debug, Clone, Copy)]
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
