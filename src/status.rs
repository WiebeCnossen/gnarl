use crate::audit::Severity;

/// Successful process outcome. Policy codes 10–14 are not [`crate::Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunStatus {
    policy_exit: u8,
}

impl RunStatus {
    pub fn ok() -> Self {
        Self { policy_exit: 0 }
    }

    pub fn from_max_ignore_severity(severity: Option<Severity>) -> Self {
        Self {
            policy_exit: match severity {
                None => 0,
                Some(Severity::Info) => 10,
                Some(Severity::Low) => 11,
                Some(Severity::Moderate) => 12,
                Some(Severity::High) => 13,
                Some(Severity::Critical) => 14,
            },
        }
    }

    pub fn policy_exit(self) -> u8 {
        self.policy_exit
    }
}

/// Without `--auto-ignore`, newly found ignore severities MUST NOT change the exit.
pub fn auto_ignore_status(auto_ignore: bool, max_new: Option<Severity>) -> RunStatus {
    if auto_ignore {
        RunStatus::from_max_ignore_severity(max_new)
    } else {
        RunStatus::ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn maps_none_and_each_severity() {
        assert_eq!(RunStatus::from_max_ignore_severity(None).policy_exit(), 0);
        assert_eq!(
            RunStatus::from_max_ignore_severity(Some(Severity::Info)).policy_exit(),
            10
        );
        assert_eq!(
            RunStatus::from_max_ignore_severity(Some(Severity::Low)).policy_exit(),
            11
        );
        assert_eq!(
            RunStatus::from_max_ignore_severity(Some(Severity::Moderate)).policy_exit(),
            12
        );
        assert_eq!(
            RunStatus::from_max_ignore_severity(Some(Severity::High)).policy_exit(),
            13
        );
        assert_eq!(
            RunStatus::from_max_ignore_severity(Some(Severity::Critical)).policy_exit(),
            14
        );
    }

    #[test]
    fn policy_codes_are_not_errors() {
        let status = RunStatus::from_max_ignore_severity(Some(Severity::High));
        let _: u8 = status.policy_exit();
        let _err: Option<Error> = None;
        assert_ne!(status.policy_exit(), 1);
    }

    #[test]
    fn without_flag_critical_suggestions_still_exit_zero() {
        assert_eq!(
            auto_ignore_status(false, Some(Severity::Critical)).policy_exit(),
            0
        );
        assert_eq!(
            auto_ignore_status(true, Some(Severity::Critical)).policy_exit(),
            14
        );
        assert_eq!(auto_ignore_status(true, None).policy_exit(), 0);
    }
}
