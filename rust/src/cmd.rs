use std::env::Args;

#[derive(Debug, Clone, Copy)]
pub struct Options {
    no_install: bool,
}

impl Options {
    fn read(mut args: Vec<String>) -> (Self, Vec<String>) {
        let mut no_install = false;
        for i in (0..args.len()).rev() {
            if args[i] == "-x" {
                no_install = true;
                args.remove(i);
            }
        }

        (Self { no_install }, args)
    }

    pub fn no_install(&self) -> bool {
        self.no_install
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Verb {
    Auto,
    Reset,
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

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }
}

impl TryFrom<Args> for Command {
    type Error = crate::Error;
    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let (options, mut parameters) = Options::read(args.skip(1).collect());

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
