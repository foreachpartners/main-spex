#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::fixtures::{check, scoped_file};
    use crate::commands::check::runners::Hit;
    use crate::rule::CheckKind;
    use tempfile::tempdir;

    fn run(pattern: &str, negate: bool, files: &[ScopedFile]) -> Vec<Hit> {
        let check = check(CheckKind::Regexp {
            pattern: pattern.to_string(),
            negate,
        });
        let files: Vec<&ScopedFile> = files.iter().collect();
        run_regexp(&check, pattern, negate, &files).expect("regexp outcome")
    }

    #[test]
    fn forbidden_pattern_reports_each_match() {
        let dir = tempdir().expect("temp dir");
        let file = scoped_file(
            &dir,
            "src/main.rs",
            "let api_key = \"superSecretValue\";\nok();\n",
        );

        let hits = run("api_key\\s*=\\s*\"[^\"]{8,}\"", false, &[file]);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].location, Some(Location::at_line("src/main.rs", 1)));
    }

    #[test]
    fn negated_pattern_reports_when_absent() {
        let dir = tempdir().expect("temp dir");
        let file = scoped_file(&dir, "src/main.rs", "fn main() {}\n");

        let hits = run("Copyright", true, &[file]);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].location, Some(Location::in_file("src/main.rs")));
    }
}
