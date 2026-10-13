#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn displays_the_name_written_in_spex_yaml() {
        assert_eq!(Layout::MultiRepo.to_string(), "multi-repo");
        assert_eq!(Layout::SingleRepo.to_string(), "single-repo");
        assert_eq!(Layout::Monorepo.to_string(), "monorepo");
    }

    #[test]
    fn deserializes_the_displayed_name() {
        for layout in [Layout::MultiRepo, Layout::SingleRepo, Layout::Monorepo] {
            let parsed: Layout =
                serde_yaml_ng::from_str(&layout.to_string()).expect("parse layout");
            assert_eq!(parsed, layout);
        }
    }

    #[test]
    fn choices_list_every_layout() {
        assert_eq!(Layout::choices(), "multi-repo, single-repo, monorepo");
    }

    #[test]
    fn defaults_to_multi_repo() {
        assert_eq!(Layout::default(), Layout::MultiRepo);
    }

    #[test]
    fn multi_repo_spex_dir_is_named_after_the_project() {
        assert_eq!(
            Layout::MultiRepo.spex_dir_name("pizza-shop"),
            "pizza-shop-spex"
        );
    }

    #[test]
    fn single_repo_and_monorepo_spex_dir_is_named_spex() {
        assert_eq!(Layout::SingleRepo.spex_dir_name("pizza-shop"), "spex");
        assert_eq!(Layout::Monorepo.spex_dir_name("pizza-shop"), "spex");
    }

    #[test]
    fn agent_root_is_the_spex_dir_only_in_multi_repo() {
        assert_eq!(Layout::MultiRepo.agent_root(), ".");
        assert_eq!(Layout::SingleRepo.agent_root(), "..");
        assert_eq!(Layout::Monorepo.agent_root(), "..");
    }
}
