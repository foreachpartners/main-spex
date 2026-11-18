# Porker pilot findings

Task `PKR-003` in the Porker Spex repository tracks these findings and their application to Porker. Tested CLI 0.38.0 from `4d3d1ee180bb2b8055b1b6410d70c4f773798a9b`, SHA-256 `38832dcc883e061f71042bcf5957628347674db2567c2ca23b54cc4978fd41de`.

## Secret rule flags boolean and literal text

In a scratch single-repository project under this worktree, define an application package pointing to the repository root. Copy `secret-handling` with `spex lib rules --id secret-handling --copy <spex-dir>/rules`, attach it to the application package, and add `smoke.js` at the package root.

```javascript
console.log(`Response received: ${!!token}`);
console.log(`Response value: ${token}`);
console.log('Token received');
console.log('Request complete');
```

`spex check` exited 1 with `no-secret-in-console-log` errors on lines 1, 2 and 3. Line 2 is the direct leak control. Line 1 logs a boolean and line 3 is static text. Line 4 produced no finding. The current rule searches all descendants of the call with a keyword regex. Treat the boolean and static message as false positives when reviewing actual secret exposure.

Porker reproduced the boolean finding on `porker-server` at `32bd8201d5a7dd9d203c9cb8fd15bcfb7404d981`, `tests/k6/scenarios/smoke.js:35`. The line interpolates the test account email and `!!body.data.token || !!body.data.access_token`. The revision-gated report records the finding. Keep this case and the direct leak control when tightening the rule.

## Receipt help and task reference behavior

Run `spex receipt --help`, then `spex receipt --project <project> --journal <scratch> append <receipt.json>`. Help lists `--project`. The command exits 2 with `this command does not use --project`. Reproduced on the shopping-notes example and on Porker.

With `--journal` alone, a valid pending-decision receipt can refer to a missing `Tasks/PKR-ARCHIVE-PROBE-MISSING.md`. Use the `context_digest` from `spex context --id api-healthcheck` on the example or `--id first-play-loop` on Porker. Set the five verification stages to `not-run` and store the path as `decision.reference`. `append` and `show` both exited 0 and the file remained absent. The [receipt spec](../../specs/cli/spex-receipt-cli-cmd.md) documents that evidence references are reported claims. Git trailers and the Markdown task record provide the archive link in the Porker workflow.
