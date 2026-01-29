use std::{env::Args, iter};

pub enum Verb {
    Auto(bool),
    Reset(Vec<String>, bool),
    Info,
    Help,
}

impl TryFrom<Args> for Verb {
    type Error = crate::Error;
    fn try_from(mut args: Args) -> Result<Self, Self::Error> {
        match args.nth(1).as_deref() {
            None => Ok(Verb::Auto(false)),
            Some("-n") => Ok(Verb::Auto(true)),
            Some("auto") => Ok(Verb::Auto(args.next().as_deref() == Some("-n"))),
            Some("reset") => Ok(Verb::reset(args)),
            Some("info") => Ok(Verb::Info),
            Some("help") => Ok(Verb::Help),
            Some(verb) => Err(crate::Error::String(format!("Unknown verb {}", verb))),
        }
    }
}

impl Verb {
    fn reset(mut args: Args) -> Self {
        match args.next().as_deref() {
            None => Verb::Reset(vec![], false),
            Some("-n") => Verb::Reset(args.collect(), true),
            Some(arg) => Verb::Reset(iter::once(arg.to_string()).chain(args).collect(), false),
        }
    }
}
