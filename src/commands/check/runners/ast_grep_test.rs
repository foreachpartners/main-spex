#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::fixtures::{check, scoped_file};
    use crate::commands::check::runners::Hit;
    use crate::rule::CheckKind;
    use tempfile::tempdir;

    fn ast_grep_check() -> Check {
        check(CheckKind::AstGrep {
            pattern: None,
            rule: None,
            language: None,
            negate: false,
        })
    }

    fn run(
        pattern: Option<&str>,
        rule_yaml: Option<&str>,
        language: Option<&str>,
        negate: bool,
        files: &[ScopedFile],
    ) -> Outcome {
        let rule: Option<Value> =
            rule_yaml.map(|yaml| serde_yaml_ng::from_str(yaml).expect("valid rule yaml"));
        let files: Vec<&ScopedFile> = files.iter().collect();
        run_ast_grep(
            &ast_grep_check(),
            pattern,
            rule.as_ref(),
            language,
            negate,
            &files,
        )
    }

    fn run_pattern(pattern: &str, files: &[ScopedFile]) -> Vec<Hit> {
        run(Some(pattern), None, None, false, files).expect("ast-grep outcome")
    }

    fn run_rule(rule_yaml: &str, files: &[ScopedFile]) -> Vec<Hit> {
        run(None, Some(rule_yaml), None, false, files).expect("ast-grep outcome")
    }

    #[test]
    fn pattern_matches_structurally_not_textually() {
        let dir = tempdir().expect("temp dir");
        let file = scoped_file(
            &dir,
            "src/main.ts",
            "// console.log(\"not real\")\nconsole.log(\"hi\");\nconsole.error(\"bye\");\n",
        );

        let hits = run_pattern("console.log($$$ARGS)", &[file]);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].location, Some(Location::at_line("src/main.ts", 2)));
    }

    #[test]
    fn infers_language_from_extension() {
        let dir = tempdir().expect("temp dir");
        let ts = scoped_file(&dir, "src/main.ts", "console.log(1);\n");
        let py = scoped_file(&dir, "src/main.py", "print(1)\n");

        let hits = run_pattern("console.log($$$ARGS)", &[ts, py]);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].location, Some(Location::at_line("src/main.ts", 1)));
    }

    #[test]
    fn reports_hits_in_file_order_across_languages() {
        let dir = tempdir().expect("temp dir");
        let files = [
            scoped_file(&dir, "src/a.ts", "trace(1);\n"),
            scoped_file(&dir, "src/b.js", "trace(1);\n"),
            scoped_file(&dir, "src/c.tsx", "trace(1);\n"),
            scoped_file(&dir, "src/d.rs", "fn run() { trace(1); }\n"),
            scoped_file(&dir, "src/e.py", "trace(1)\n"),
        ];

        let hits = run_pattern("trace($$$ARGS)", &files);

        let locations: Vec<_> = hits.iter().map(|hit| hit.location.clone()).collect();
        assert_eq!(
            locations,
            [
                Some(Location::at_line("src/a.ts", 1)),
                Some(Location::at_line("src/b.js", 1)),
                Some(Location::at_line("src/c.tsx", 1)),
                Some(Location::at_line("src/d.rs", 1)),
                Some(Location::at_line("src/e.py", 1)),
            ]
        );
    }

    #[test]
    fn negated_pattern_reports_when_absent() {
        let dir = tempdir().expect("temp dir");
        let file = scoped_file(&dir, "src/main.ts", "console.error(1);\n");

        let hits =
            run(Some("console.log($$$ARGS)"), None, None, true, &[file]).expect("ast-grep outcome");

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].location, Some(Location::in_file("src/main.ts")));
    }

    #[test]
    fn rule_supports_constraints_on_metavariables() {
        let dir = tempdir().expect("temp dir");
        let file = scoped_file(
            &dir,
            "src/main.ts",
            "console.log(userId);\nconsole.log(apiToken);\n",
        );

        let hits = run_rule(
            r#"
rule:
  pattern: console.log($ARG)
constraints:
  ARG:
    regex: "(?i)token|secret|password"
"#,
            &[file],
        );

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].location, Some(Location::at_line("src/main.ts", 2)));
    }

    #[test]
    fn invalid_language_is_an_outcome_error() {
        let dir = tempdir().expect("temp dir");
        let file = scoped_file(&dir, "src/main.ts", "console.log(1);\n");

        let outcome = run(
            Some("console.log($$$ARGS)"),
            None,
            Some("not-a-real-language"),
            false,
            &[file],
        );

        let message = outcome.map(|_| ()).expect_err("invalid language");
        assert!(message.starts_with("unknown ast-grep language `not-a-real-language`"));
    }

    #[test]
    fn rule_constraint_on_multi_metavariable() {
        let dir = tempdir().expect("temp dir");
        let file = scoped_file(
            &dir,
            "src/main.ts",
            "console.log(userId);\nconsole.log(\"token=\", apiToken);\n",
        );

        let hits = run_rule(
            r#"
rule:
  all:
    - pattern: console.$METHOD($$$ARGS)
    - has:
        stopBy: end
        regex: "(?i)(token|secret|password|credential|api[_-]?key)"
"#,
            &[file],
        );

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].location, Some(Location::at_line("src/main.ts", 2)));
    }

    #[test]
    fn rule_supports_any_composite_with_constraints() {
        let dir = tempdir().expect("temp dir");
        let file = scoped_file(&dir, "src/main.rs", "info!(user_id);\nwarn!(api_token);\n");

        let hits = run(
            None,
            Some(
                r#"
rule:
  all:
    - any:
        - pattern: info!($$$ARGS)
        - pattern: warn!($$$ARGS)
        - pattern: error!($$$ARGS)
    - has:
        stopBy: end
        regex: "(?i)(token|secret|password|credential|api[_-]?key)"
"#,
            ),
            Some("rust"),
            false,
            &[file],
        )
        .expect("ast-grep outcome");

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].location, Some(Location::at_line("src/main.rs", 2)));
    }

    #[test]
    fn rule_constraint_only_applies_when_var_is_bound() {
        let dir = tempdir().expect("temp dir");
        let file = scoped_file(
            &dir,
            "src/main.test.ts",
            "fit('runs', () => {});\ndescribe.only('suite', () => {});\narr.only('nope', () => {});\n",
        );

        let hits = run_rule(
            r#"
rule:
  any:
    - pattern: fit($$$ARGS)
    - pattern: fdescribe($$$ARGS)
    - pattern: $OBJ.only($$$ARGS)
constraints:
  OBJ:
    regex: "^(describe|it|test|context)$"
"#,
            &[file],
        );

        let mut lines: Vec<_> = hits
            .iter()
            .map(|hit| hit.location.clone().expect("location").line)
            .collect();
        lines.sort();
        assert_eq!(lines, vec![Some(1), Some(2)]);
    }
}
