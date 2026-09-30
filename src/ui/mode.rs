use crate::cmd::Verb;

/// How gnarl should present output for this process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    /// Tagged stdout (historical behavior).
    Stdout,
    /// Interactive ratatui surfaces.
    Interactive,
}

/// Interactive iff TTY, not `--raw`, and verb is `auto` or `check`.
pub fn select_ui_mode(is_tty: bool, raw: bool, verb: Verb) -> UiMode {
    match verb {
        Verb::Auto | Verb::Check if is_tty && !raw => UiMode::Interactive,
        _ => UiMode::Stdout,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interactive_only_for_auto_check_on_tty_without_raw() {
        assert_eq!(
            select_ui_mode(true, false, Verb::Auto),
            UiMode::Interactive
        );
        assert_eq!(
            select_ui_mode(true, false, Verb::Check),
            UiMode::Interactive
        );
        assert_eq!(select_ui_mode(true, true, Verb::Auto), UiMode::Stdout);
        assert_eq!(select_ui_mode(false, false, Verb::Auto), UiMode::Stdout);
        assert_eq!(select_ui_mode(true, false, Verb::Reset), UiMode::Stdout);
        assert_eq!(select_ui_mode(true, false, Verb::Help), UiMode::Stdout);
        assert_eq!(select_ui_mode(true, false, Verb::Info), UiMode::Stdout);
    }
}
