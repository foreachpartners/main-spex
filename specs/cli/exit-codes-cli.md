# Exit codes and error output

Every command reports its outcome the same way, so a CI script can tell a tool failure from a finding.

## Acceptance criteria

- MUST exit `0` on success.
- MUST exit `1` when the command ran and reported failures: `check` errors, `coverage` gaps, `check-schema` violations.
- MUST exit `2` when the command could not run: invalid arguments, a missing `spex.yaml`, an unreadable schema.
- MUST print one `error: <message>` line to stderr when a command could not run.
- MUST follow the `error: <message>` line with a detailed report when `RUST_BACKTRACE` is set: the source location where the error arose, each added context with its location, and a backtrace.
