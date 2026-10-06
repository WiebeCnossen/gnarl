use std::env::Args;
use std::io::IsTerminal;

use crate::audit::Severity;
use crate::ui::{UiMode, select_ui_mode};

#[derive(Debug, Clone, Copy)]
pub struct Options {
    install_on_change: bool,
    severity: Severity,
    raw: bool,
    auto_ignore: bool,
}

impl Options {
    fn read(args: &mut Vec<String>) -> Result<Self, crate::Error> {
        let mut install_on_change = false;
        let mut severity = Severity::Info;
        let mut raw = false;
        let mut auto_ignore = false;
        for i in (0..args.len()).rev() {
            match args[i].as_str() {
                "-x" | "--install-on-change" => {
                    install_on_change = true;
                    args.remove(i);
                }
                "--raw" => {
                    raw = true;
                    args.remove(i);
                }
                "--auto-ignore" => {
                    auto_ignore = true;
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
            install_on_change,
            severity,
            raw,
            auto_ignore,
        })
    }

    pub fn install_on_change(&self) -> bool {
        self.install_on_change
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn raw(&self) -> bool {
        self.raw
    }

    pub fn auto_ignore(&self) -> bool {
        self.auto_ignore
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

    fn assemble(
        verb: Verb,
        options: Options,
        parameters: Vec<String>,
    ) -> Result<Self, crate::Error> {
        if options.auto_ignore && verb != Verb::Auto {
            return Err("--auto-ignore is only valid with auto".into());
        }
        Ok(Self {
            verb,
            options,
            parameters,
        })
    }
}

impl TryFrom<Args> for Command {
    type Error = crate::Error;
    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let mut parameters = args.skip(1).collect::<Vec<String>>();
        let options = Options::read(&mut parameters)?;

        if parameters.is_empty() {
            return Self::assemble(Verb::Auto, options, parameters);
        }

        let verb = match parameters.remove(0).as_str() {
            "auto" => Verb::Auto,
            "reset" => Verb::Reset,
            "check" => Verb::Check,
            "info" => Verb::Info,
            "help" => Verb::Help,
            unknown => return Err(format!("Unknown verb {}", unknown).into()),
        };

        Self::assemble(verb, options, parameters)
    }
}

/// Tagged-stdout help lines (without version banner).
pub fn help_lines() -> &'static [&'static str] {
    &[
        "the yarn v4 companion tool",
        "usage: gnarl [<auto | reset | check | info | help> <args>]",
        "> gnarl [auto] [--raw] [--auto-ignore] [-x|--install-on-change] [-s <severity>]",
        "> gnarl reset [--raw] [-x|--install-on-change] package-names...",
        "> gnarl check [--raw]",
        "> gnarl info",
        "> gnarl help",
        "--raw  force tagged stdout even on a TTY",
        "--auto-ignore  on auto: persist suggested ignores; policy exit is max new-ignore severity",
        "-x, --install-on-change  skip opening install+dedupe; still refresh after this-run package.json or yarn.lock changes",
        "exit 0   success (no new ignores, or without --auto-ignore)",
        "exit 1   tool error",
        "exit 10  auto --auto-ignore: max new ignore is info",
        "exit 11  auto --auto-ignore: max new ignore is low",
        "exit 12  auto --auto-ignore: max new ignore is moderate",
        "exit 13  auto --auto-ignore: max new ignore is high",
        "exit 14  auto --auto-ignore: max new ignore is critical",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Command {
        parse_result(args).unwrap()
    }

    fn parse_result(args: &[&str]) -> Result<Command, crate::Error> {
        let mut parameters: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
        let options = Options::read(&mut parameters)?;
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
        Command::assemble(verb, options, parameters)
    }

    #[test]
    fn parses_raw_with_other_flags() {
        let cmd = parse(&["check", "--raw", "-x", "-s", "high"]);
        assert_eq!(cmd.verb(), Verb::Check);
        assert!(cmd.options().raw());
        assert!(cmd.options().install_on_change());
        assert_eq!(cmd.options().severity(), Severity::High);
        assert!(!cmd.options().auto_ignore());
    }

    #[test]
    fn raw_defaults_false() {
        let cmd = parse(&["auto"]);
        assert!(!cmd.options().raw());
        assert!(!cmd.options().auto_ignore());
        assert!(!cmd.options().install_on_change());
    }

    #[test]
    fn parses_install_on_change_short_on_auto() {
        let cmd = parse(&["auto", "-x"]);
        assert_eq!(cmd.verb(), Verb::Auto);
        assert!(cmd.options().install_on_change());
    }

    #[test]
    fn parses_install_on_change_long_on_auto() {
        let cmd = parse(&["auto", "--install-on-change"]);
        assert_eq!(cmd.verb(), Verb::Auto);
        assert!(cmd.options().install_on_change());
    }

    #[test]
    fn parses_install_on_change_on_reset() {
        let cmd = parse(&["reset", "--install-on-change", "lodash"]);
        assert_eq!(cmd.verb(), Verb::Reset);
        assert!(cmd.options().install_on_change());
        assert_eq!(cmd.parameters(), &["lodash".to_owned()]);
    }

    #[test]
    fn default_auto_does_not_set_install_on_change() {
        let cmd = parse(&[]);
        assert_eq!(cmd.verb(), Verb::Auto);
        assert!(!cmd.options().install_on_change());
    }

    #[test]
    fn parses_auto_ignore_on_default_auto() {
        let cmd = parse(&["--auto-ignore"]);
        assert_eq!(cmd.verb(), Verb::Auto);
        assert!(cmd.options().auto_ignore());
        assert!(!cmd.options().raw());
    }

    #[test]
    fn parses_auto_ignore_with_raw() {
        let cmd = parse(&["auto", "--raw", "--auto-ignore"]);
        assert_eq!(cmd.verb(), Verb::Auto);
        assert!(cmd.options().auto_ignore());
        assert!(cmd.options().raw());
    }

    #[test]
    fn rejects_auto_ignore_on_check() {
        let err = match parse_result(&["check", "--auto-ignore"]) {
            Err(e) => e,
            Ok(_) => panic!("expected error"),
        };
        assert!(err.to_string().contains("--auto-ignore"));
    }

    #[test]
    fn rejects_auto_ignore_on_reset() {
        let err = match parse_result(&["reset", "--auto-ignore", "lodash"]) {
            Err(e) => e,
            Ok(_) => panic!("expected error"),
        };
        assert!(err.to_string().contains("--auto-ignore"));
    }

    #[test]
    fn help_lines_document_auto_ignore_and_policy_exits() {
        let help = help_lines().join("\n");
        assert!(help.contains("--auto-ignore"));
        assert!(help.contains("exit 0   success"));
        assert!(help.contains("exit 1   tool error"));
        assert!(help.contains("exit 10  auto --auto-ignore: max new ignore is info"));
        assert!(help.contains("exit 11  auto --auto-ignore: max new ignore is low"));
        assert!(help.contains("exit 12  auto --auto-ignore: max new ignore is moderate"));
        assert!(help.contains("exit 13  auto --auto-ignore: max new ignore is high"));
        assert!(help.contains("exit 14  auto --auto-ignore: max new ignore is critical"));
        assert!(help.contains("--raw"));
        assert!(help.contains("-x"));
        assert!(help.contains("--install-on-change"));
        assert!(help.contains("skip opening install+dedupe"));
        assert!(help.contains("package.json"));
        assert!(help.contains("yarn.lock"));
    }

    #[test]
    fn readme_documents_bot_best_practice() {
        let readme = include_str!("../README.md");
        assert!(readme.contains("gnarl auto --raw --auto-ignore --install-on-change"));
        assert!(readme.contains("--install-on-change"));
        assert!(readme.contains("`-x`"));
    }
}
