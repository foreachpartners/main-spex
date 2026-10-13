# `spex jq`

Runs a [jq](https://jqlang.org/manual/) filter on JSON or YAML data, so a script or an AI agent reads a field of `spex info`, `spex.yaml`, a spec, or a rule without installing `jq` or `yq`. The filter language is jq's, run by the embedded [jaq](https://gedenkt.at/jaq/manual/) engine, whose manual lists its differences from jq. Unlike most commands it does not look for `spex.yaml`; it reads only the given file or standard input.

```sh
spex jq [OPTIONS] <FILTER> [FILE]
```

- `<FILTER>` — the jq filter, e.g. `.project_name` or `.packages[].path`.
- `[FILE]` — the JSON or YAML file to read; standard input when omitted. The input is read as JSON when it is valid JSON, otherwise as YAML, and holds a single document.
- Prints every result the filter produces on its own line, as indented JSON.
- Exits `0` when the filter ran. Exits `2` when it could not run: an invalid filter, an unreadable or unparsable input, or a filter that fails at runtime, such as indexing a number.

## Flags

- `--raw-output` (`-r`) — print a string result without quotes or escaping, so a script can capture it directly.
- `--compact-output` (`-c`) — print each result on a single line.

```sh
spex jq -r .project_name spex.yaml
spex info | spex jq -r '.packages[].path'
```
