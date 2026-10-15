use crate::support::testing::{package, spex_dir};

use super::*;

mod report_target_tests {
    use super::*;

    #[test]
    fn a_package_target_is_named_and_based_at_the_package() {
        let api = package("api", "services/api");
        let target = ReportTarget::Package(&api);

        assert_eq!(target.name(), "api");
        assert_eq!(target.base_dir(), spex_dir().join("services/api"));
    }

    #[test]
    fn the_project_target_is_based_at_the_spex_dir() {
        let target = ReportTarget::Project {
            spex_dir: spex_dir(),
        };

        assert_eq!(target.name(), "project");
        assert_eq!(target.base_dir(), spex_dir());
    }
}

mod location_url_tests {
    use super::*;

    #[test]
    fn joins_the_base_dir_with_the_location() {
        let base = Path::new("/project/services/api");

        assert_eq!(
            location_url(base, &Location::in_file("CLAUDE.md")),
            "file:///project/services/api/CLAUDE.md"
        );
    }

    #[test]
    fn drops_a_trailing_line_number() {
        let base = Path::new("/project/services/api");

        assert_eq!(
            location_url(base, &Location::at_line("CLAUDE.md", 50)),
            "file:///project/services/api/CLAUDE.md"
        );
    }

    #[test]
    fn encodes_spaces_in_the_path() {
        let base = Path::new("/project/my package");

        assert_eq!(
            location_url(base, &Location::in_file("CLAUDE.md")),
            "file:///project/my%20package/CLAUDE.md"
        );
    }
}
