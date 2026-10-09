use crate::support::testing::{repository, spex_dir};

use super::*;

mod report_target_tests {
    use super::*;

    #[test]
    fn a_repository_target_is_named_and_based_at_the_repository() {
        let api = repository("api", "services/api");
        let target = ReportTarget::Repository(&api);

        assert_eq!(target.name(), "api");
        assert_eq!(target.base_dir(), spex_dir().join("services/api"));
    }

    #[test]
    fn the_workspace_target_is_based_at_the_spex_dir() {
        let target = ReportTarget::Workspace {
            spex_dir: spex_dir(),
        };

        assert_eq!(target.name(), "workspace");
        assert_eq!(target.base_dir(), spex_dir());
    }

    #[test]
    fn only_the_spex_repository_target_is_the_spex_repo() {
        let api = repository("api", "services/api");
        let spex_repo = Repository {
            is_spex_repo: true,
            ..repository("demo-spex", ".")
        };

        assert!(ReportTarget::Repository(&spex_repo).is_spex_repo());
        assert!(!ReportTarget::Repository(&api).is_spex_repo());
        assert!(
            !ReportTarget::Workspace {
                spex_dir: spex_dir()
            }
            .is_spex_repo()
        );
    }
}

mod location_url_tests {
    use super::*;

    #[test]
    fn joins_the_base_dir_with_the_location() {
        let base = Path::new("/workspace/services/api");

        assert_eq!(
            location_url(base, "CLAUDE.md"),
            "file:///workspace/services/api/CLAUDE.md"
        );
    }

    #[test]
    fn drops_a_trailing_line_number() {
        let base = Path::new("/workspace/services/api");

        assert_eq!(
            location_url(base, "CLAUDE.md:50"),
            "file:///workspace/services/api/CLAUDE.md"
        );
    }

    #[test]
    fn encodes_spaces_in_the_path() {
        let base = Path::new("/workspace/my repo");

        assert_eq!(
            location_url(base, "CLAUDE.md"),
            "file:///workspace/my%20repo/CLAUDE.md"
        );
    }
}
