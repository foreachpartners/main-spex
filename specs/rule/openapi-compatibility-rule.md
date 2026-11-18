# OpenAPI compatibility

The optional `openapi-compatibility` library rule checks consumers' OpenAPI contracts against an explicit earlier checkout.

## Acceptance criteria

- MUST run this rule only in packages that enable it. Its per-file check receives matching OpenAPI filenames from those packages and any matching contracts they declare.
- MUST require `git` and `oasdiff` on `PATH` and stop the script check on its configured timeout.
- MUST take the comparison baseline from `SPEX_OPENAPI_BASELINE_DIR`, an absolute directory containing an earlier checkout or export of the repository that stores the contract.
- MUST resolve the repository that stores the current contract even when the declaring package passes it with a parent-relative path. Compare it with the matching repository-relative path under the baseline directory.
- MUST invoke `oasdiff breaking` with the baseline file first and the current file second. Use both files at their real paths so local relative references resolve in their own trees. Fail at oasdiff warning severity or higher and explicitly disable its optional fetch and share operations.
- MUST report breaking changes and inability to compare as findings on the current contract. Report a missing baseline directory, missing baseline contract or a baseline equal to the current checkout clearly.
- MUST leave both checkouts unchanged. Baseline selection does not fetch or install anything.

## Example

Copy the rule with `spex lib rules --id openapi-compatibility --copy rules` from the spex directory, then add `openapi-compatibility` to the declaring package's rules. Run the check after preparing the approved baseline.

```sh
SPEX_OPENAPI_BASELINE_DIR=/work/approved-api-baseline spex check --package api
```

Run comparisons with trusted contract files and their reference trees. For contributions from untrusted authors, isolate the runner from the network and sensitive local files because oasdiff resolves external references. [oasdiff reference loading](https://github.com/oasdiff/oasdiff/blob/main/docs/EXTERNAL-REFS.md)
