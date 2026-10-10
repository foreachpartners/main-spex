#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::commands::check::fixtures::{check, scoped_file};
    use crate::commands::check::runners::Hit;
    use crate::rule::CheckKind;
    use tempfile::tempdir;

    fn run(pattern: &str, negate: bool, files: &[ScopedFile]) -> Vec<Hit> {
        let check = check(CheckKind::Glob {
            pattern: pattern.to_string(),
            negate,
        });
        let files: Vec<&ScopedFile> = files.iter().collect();
        run_glob(&check, pattern, negate, &files).expect("glob outcome")
    }

    #[test]
    fn forbidden_pattern_reports_matching_paths() {
        let dir = tempdir().expect("temp dir");
        let bak = scoped_file(&dir, "notes.bak", "");
        let src = scoped_file(&dir, "src/main.rs", "");

        let hits = run("**/*.bak", false, &[bak, src]);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].location, Some(Location::in_file("notes.bak")));
    }

    #[test]
    fn negated_pattern_reports_when_required_path_absent() {
        let dir = tempdir().expect("temp dir");
        let src = scoped_file(&dir, "src/main.rs", "");

        let hits = run("**/LICENSE", true, &[src]);

        assert_eq!(hits.len(), 1);
        assert!(hits[0].location.is_none());
    }
}
