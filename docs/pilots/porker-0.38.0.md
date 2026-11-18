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

## Rule scripts look up the CLI on PATH

In a scratch Git checkout inside the pilot worktree, initialize a single-repository project with a root package that applies `commit-message-format`. Commit the scaffold with `chore`, then add a source annotation for an existing spec and commit it as `feat` with a matching `@spec` tag. Invoke the pinned executable by absolute path.

- With `PATH=/usr/bin:/bin`, `spex check` prints `commit-spec-id-trace.sh: 88: spex: not found` as a warning and exits 0. The script has not checked the commit against the spec catalog.
- With the same executable directory prepended to `PATH`, the same checkout and rule exit 0 with no findings.

Porker also exercised `openapi-conventions`. That script invokes `spex jq` under inherited `PATH` and reported `could not read the vacuum report` without the CLI directory, although Vacuum had returned valid JSON. With the pin on `PATH`, the rule passed and a direct Vacuum run returned `[]`. The initial and recovered reports are in the PKR-002 office run.

A framework regression test should invoke `check` by absolute path with a different or missing `spex` on `PATH`. Bundled scripts need a reliable way to invoke the executable running the check.

## An inline Git tag in a task is read as a source annotation

Add a tracked `task.md` to a configured application package in a scratch single-repository project. Write a sentence quoting the Git-message tag `@spec first-play-loop` in inline code. The pinned CLI's `check` exits 1 with `spex-internal-spec-id-tracing/annotation-format` at that line. Change the sentence to “The commit contains the first-play-loop specification tag.” The same check exits 0.

Porker produced the former result at `Tasks/PKR-002.md:63` on `93be0d4`. A task may cite the commit's tag without creating a source-level trace. The archive now links to the commit and names the spec. The original report is retained under `runs/PKR-002/control-task-inline-tag.log`. A follow-up can distinguish quoted evidence in a task record from executable trace locations.
