#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::error::describe;

    const PRETTY: Output = Output {
        raw: false,
        compact: false,
    };
    const RAW: Output = Output {
        raw: true,
        compact: false,
    };
    const COMPACT: Output = Output {
        raw: false,
        compact: true,
    };

    fn evaluated(filter: &str, text: &str, output: Output) -> String {
        let mut out = Vec::new();
        evaluate(filter, text, output, &mut out).expect("evaluate");
        String::from_utf8(out).expect("utf-8 output")
    }

    fn failure(filter: &str, text: &str) -> String {
        let mut out = Vec::new();
        let report = evaluate(filter, text, PRETTY, &mut out).expect_err("evaluate should fail");
        describe(&report)
    }

    #[test]
    fn quotes_string_results_unless_raw() {
        let json = r#"{"project_name": "shopping-notes"}"#;

        assert_eq!(
            evaluated(".project_name", json, PRETTY),
            "\"shopping-notes\"\n"
        );
        assert_eq!(evaluated(".project_name", json, RAW), "shopping-notes\n");
    }

    #[test]
    fn reads_yaml_input() {
        let yaml =
            "project_name: shopping-notes\npackages:\n  - path: ../api\n  - path: ../frontend\n";

        assert_eq!(
            evaluated(".packages[].path", yaml, RAW),
            "../api\n../frontend\n"
        );
    }

    #[test]
    fn reads_json_that_yaml_cannot_express() {
        let json = "{\n\t\"emoji\": \"\\ud83d\\ude00\"\n}\n";

        assert_eq!(evaluated(".emoji", json, RAW), "\u{1f600}\n");
    }

    #[test]
    fn prints_each_result_on_its_own_line() {
        assert_eq!(evaluated(".[]", "[1, 2, 3]", PRETTY), "1\n2\n3\n");
    }

    #[test]
    fn indents_objects_unless_compact() {
        let json = r#"{"a": {"b": 1}}"#;

        assert_eq!(
            evaluated(".", json, PRETTY),
            "{\n  \"a\": {\n    \"b\": 1\n  }\n}\n"
        );
        assert_eq!(evaluated(".", json, COMPACT), "{\"a\":{\"b\":1}}\n");
    }

    #[test]
    fn raw_output_leaves_non_strings_as_json() {
        assert_eq!(
            evaluated(".a", r#"{"a": [1, "x"]}"#, RAW),
            "[\n  1,\n  \"x\"\n]\n"
        );
    }

    #[test]
    fn rejects_an_invalid_filter() {
        let message = failure(".a[", "{}");

        assert!(message.starts_with("invalid jq filter: "), "{message}");
        assert!(!message.contains('\n'), "{message}");
    }

    #[test]
    fn rejects_a_filter_that_fails_at_runtime() {
        let message = failure(".a.b", r#"{"a": 1}"#);

        assert!(message.starts_with("jq: "), "{message}");
    }

    #[test]
    fn rejects_input_that_is_not_json_or_yaml() {
        let message = failure(".", "{unclosed: [");

        assert!(
            message.starts_with("could not parse the input as JSON or YAML"),
            "{message}"
        );
    }
}
