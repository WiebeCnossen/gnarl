use std::env::Args;

pub enum Verb {
    Auto(bool),
    Reset(Vec<String>, bool),
    Help,
}

impl TryFrom<Args> for Verb {
    type Error = crate::Error;
    fn try_from(mut args: Args) -> Result<Self, Self::Error> {
        match args.next().map(|s| s.as_str()) {
            Some("auto") => Ok(Verb::auto(args)),
            Some("reset") => Ok(Verb::reset(args)),
            Some("help") => Ok(Verb::Help),
            Some(verb) => Err(crate::Error::String(format!("Unknown verb {}", verb))),
            None => Ok(Verb::Auto(false)),
        }
    }
}

impl Verb {
    fn auto(mut args: Args) -> Self {
        let no_install = args.any(|arg| arg == "--no-install");
        Verb::Auto(no_install)
    }

    fn reset(args: Args) -> Self {
        let mut packages = vec![];
        let mut no_install = false;

        for arg in args {
            if arg == "--no-install" {
                no_install = true;
            } else {
                packages.push(arg);
            }
        }

        Verb::Reset(packages, no_install)
    }
}
