#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::arch_spec::{Check, CheckKind, Severity};
    use tempfile::{TempDir, tempdir};

    fn check(kind: CheckKind, pattern: Option<&str>) -> Check {
        Check {
            name: "sample".to_string(),
            description: None,
            kind,
            pattern: pattern.map(str::to_string),
            negate: false,
            include: Vec::new(),
            exclude: Vec::new(),
            severity: Severity::Error,
        }
    }

    fn scoped(dir: &TempDir, rel: &str, contents: &str) -> ScopedFile {
        let abs = dir.path().join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(&abs, contents).expect("write file");
        ScopedFile {
            abs,
            rel: PathBuf::from(rel),
        }
    }

    #[test]
    fn provided_files_resolves_existing_relative_paths() {
        let dir = tempdir().expect("temp dir");
        scoped(&dir, "src/api/main.rs", "");
        scoped(&dir, "README.md", "");

        let files = vec![
            "src/api/main.rs".to_string(),
            "./README.md".to_string(),
            "missing.rs".to_string(),
        ];
        let resolved = provided_files(dir.path(), &files);
        let names: Vec<String> = resolved.iter().map(rel_str).collect();

        assert_eq!(
            names,
            vec!["src/api/main.rs".to_string(), "README.md".to_string()]
        );
    }

    #[test]
    fn build_globset_returns_none_for_empty() {
        assert!(build_globset(&[]).expect("empty globset").is_none());
    }

    #[test]
    fn build_globset_matches_patterns() {
        let set = build_globset(&["**/*.rs".to_string()])
            .expect("valid globset")
            .expect("some globset");
        assert!(set.is_match(Path::new("src/main.rs")));
        assert!(!set.is_match(Path::new("src/main.ts")));
    }

    #[test]
    fn regexp_forbidden_reports_each_match() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(
            &dir,
            "src/main.rs",
            "let api_key = \"superSecretValue\";\nok();\n",
        );
        let check = check(CheckKind::Regexp, Some("api_key\\s*=\\s*\"[^\"]{8,}\""));

        let mut findings = Vec::new();
        run_regexp("ARCH-SECRETS", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.rs:1"));
    }

    #[test]
    fn regexp_negate_reports_when_absent() {
        let dir = tempdir().expect("temp dir");
        let file = scoped(&dir, "src/main.rs", "fn main() {}\n");
        let mut check = check(CheckKind::Regexp, Some("Copyright"));
        check.negate = true;

        let mut findings = Vec::new();
        run_regexp("ARCH-LICENSE", &check, &[&file], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("src/main.rs"));
    }

    #[test]
    fn glob_forbidden_reports_matching_paths() {
        let dir = tempdir().expect("temp dir");
        let bak = scoped(&dir, "notes.bak", "");
        let src = scoped(&dir, "src/main.rs", "");
        let check = check(CheckKind::Glob, Some("**/*.bak"));

        let mut findings = Vec::new();
        run_glob("ARCH-SECRETS", &check, &[&bak, &src], &mut findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location.as_deref(), Some("notes.bak"));
    }

    #[test]
    fn glob_negate_reports_when_required_absent() {
        let dir = tempdir().expect("temp dir");
        let src = scoped(&dir, "src/main.rs", "");
        let mut check = check(CheckKind::Glob, Some("**/LICENSE"));
        check.negate = true;

        let mut findings = Vec::new();
        run_glob("ARCH-LICENSE", &check, &[&src], &mut findings);

        assert_eq!(findings.len(), 1);
        assert!(findings[0].location.is_none());
    }

    #[test]
    fn resolve_script_prefers_repo_over_spex() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(repo.join(CHECKS_DIR)).expect("repo checks");
        std::fs::create_dir_all(spex.join(CHECKS_DIR)).expect("spex checks");
        std::fs::write(repo.join(CHECKS_DIR).join("lint.sh"), "").expect("repo script");
        std::fs::write(spex.join(CHECKS_DIR).join("lint.sh"), "").expect("spex script");

        let resolved = resolve_script("lint", &repo, &spex).expect("resolved");
        assert_eq!(resolved, repo.join(CHECKS_DIR).join("lint.sh"));
    }

    #[test]
    fn resolve_script_falls_back_to_spex() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(spex.join(CHECKS_DIR)).expect("spex checks");
        std::fs::write(spex.join(CHECKS_DIR).join("lint.sh"), "").expect("spex script");

        let resolved = resolve_script("lint", &repo, &spex).expect("resolved");
        assert_eq!(resolved, spex.join(CHECKS_DIR).join("lint.sh"));
    }

    #[test]
    fn resolve_script_ignores_extensionless_files() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(spex.join(CHECKS_DIR)).expect("spex checks");
        std::fs::write(spex.join(CHECKS_DIR).join("lint"), "").expect("spex script");

        assert!(resolve_script("lint", &repo, &spex).is_none());
    }

    #[test]
    fn resolve_script_returns_none_when_missing() {
        let workspace = tempdir().expect("workspace");
        let repo = workspace.path().join("api");
        let spex = workspace.path().join("spex");
        std::fs::create_dir_all(&repo).expect("repo");
        std::fs::create_dir_all(&spex).expect("spex");

        assert!(resolve_script("lint", &repo, &spex).is_none());
    }

    #[test]
    fn scope_files_applies_include_and_exclude() {
        let dir = tempdir().expect("temp dir");
        let rs = scoped(&dir, "src/main.rs", "");
        let ts = scoped(&dir, "src/app.ts", "");
        let test = scoped(&dir, "src/main_test.rs", "");
        let files = vec![rs, ts, test];

        let mut check = check(CheckKind::Regexp, Some("x"));
        check.include = vec!["**/*.rs".to_string()];
        check.exclude = vec!["**/*_test.rs".to_string()];

        let scoped = scope_files(&check, &files).expect("scoped");
        let names: Vec<String> = scoped.iter().map(|file| rel_str(file)).collect();
        assert_eq!(names, vec!["src/main.rs".to_string()]);
    }
}
