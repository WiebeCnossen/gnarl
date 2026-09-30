# Spec Delta

## MODIFIED Requirements

### Requirement: Blocked-by messages are unique across auto iterations

When running `auto`, gnarl MUST present each distinct `{package} blocked by {other-package}@{version}` info message at most once per program run on the activity presentation channel (tagged stdout under raw/non-TTY, or the interactive dashboard activity log under the TTY UI). If the same package / other-package / version combination would be logged again on a later outer-loop iteration of the same run, gnarl MUST NOT present it again on that channel. Fix behavior (including parent escalation and resets) MUST remain unchanged.

#### Scenario: Same blocked message suppressed on later iteration

- **WHEN** `auto` presents `{package} blocked by {other-package}@{version}` during one outer-loop iteration, and a later iteration of the same run encounters the same package, other-package, and version for a blocked non-npm dependent
- **THEN** gnarl MUST NOT present that message again on the activity channel

#### Scenario: Different blocked message still prints

- **WHEN** `auto` has already presented a blocked-by message for one package / other-package / version combination, and later encounters a blocked non-npm dependent with a different package, other-package, or version
- **THEN** gnarl MUST present the new blocked-by message on the activity channel

#### Scenario: Within-pass duplicates are not required to be suppressed

- **WHEN** a single `fix()` call would emit the same blocked-by message more than once because multiple dependents map to the same package / other-package / version
- **THEN** gnarl MAY present the message more than once within that pass (cross-iteration uniqueness is required; within-pass uniqueness is not)
