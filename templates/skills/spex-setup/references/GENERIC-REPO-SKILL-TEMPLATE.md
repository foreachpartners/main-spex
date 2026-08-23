# General

Run `spex info --repo <repository-short-name>` for general repository information and applicable rules.

# Paths

- `src/routes` - route definitions
- `src/components` - reusable UI components, one subfolder per component
- `src/lib` - shared logic: auth, config, validation, error handling
- `tests/e2e` - end-to-end tests

# Commands

Run from the `<repository-name>` repository root.

- `npm dev` - serve the app
- `npm test` - run unit tests
- `npm lint` - run linters
- `npm format` - format files

# Rules

After all changes run `npm format`, `npm lint` and `npm test`.
