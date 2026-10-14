# `spex jq`

Runs a [jq](https://jqlang.org/manual/) filter on JSON or YAML data, so a script or an AI agent reads a field of `spex info`, `spex.yaml`, a spec, or a rule without installing `jq` or `yq`.

```sh
spex jq [OPTIONS] <FILTER> [FILE]
```

## Acceptance criteria

- MUST take `<FILTER>`, the jq filter, e.g. `.project_name` or `.packages[].path`.
- MUST read the JSON or YAML file `[FILE]`, or standard input when it is omitted.
- MUST read the input as JSON when it is valid JSON, otherwise as YAML, and expect a single document.
- MUST run the filter language of jq with the embedded [jaq](https://gedenkt.at/jaq/manual/) engine, whose manual lists its differences from jq.
- MUST print every result the filter produces on its own line, as indented JSON.
- MUST exit `0` when the filter ran.
- MUST exit `2` when the filter could not run: an invalid filter, an unreadable or unparsable input, or a filter that fails at runtime, such as indexing a number.
- MUST print a string result without quotes or escaping with `--raw-output` (`-r`), so a script can capture it directly.
- MUST print each result on a single line with `--compact-output` (`-c`).
- MUST NOT look for `spex.yaml`: it reads only the given file or standard input.

## Examples

```sh
spex jq -r .project_name spex.yaml
spex info | spex jq -r '.packages[].path'
```
