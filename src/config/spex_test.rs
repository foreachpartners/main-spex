#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::support::testing;

    fn spex() -> Spex {
        testing::spex(vec![
            testing::repository("api", "../api"),
            testing::repository("web", "../web"),
        ])
    }

    #[test]
    fn validate_scopes_accepts_no_scopes() {
        assert!(spex().validate_scopes(&[]).is_ok());
    }

    #[test]
    fn validate_scopes_accepts_known_repository_short_names() {
        let scopes = ["api".to_string(), "web".to_string()];
        assert!(spex().validate_scopes(&scopes).is_ok());
    }

    #[test]
    fn validate_scopes_rejects_the_first_unknown_short_name() {
        let scopes = ["api".to_string(), "nope".to_string(), "other".to_string()];
        let error = spex().validate_scopes(&scopes).expect_err("unknown scope");
        assert!(matches!(
            error.current_context(),
            CliError::UnknownScope(name) if name == "nope"
        ));
    }

    fn selected_short_names(names: &[&str]) -> Vec<String> {
        let names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        spex()
            .select_repositories(&names)
            .expect("select")
            .iter()
            .map(|repo| repo.repository_short_name.clone())
            .collect()
    }

    #[test]
    fn select_repositories_without_names_returns_every_repository() {
        assert_eq!(selected_short_names(&[]), vec!["api", "web"]);
    }

    #[test]
    fn select_repositories_keeps_spex_yaml_order_and_matches_name_or_short_name() {
        assert_eq!(selected_short_names(&["web", "api"]), vec!["api", "web"]);
        assert_eq!(selected_short_names(&["web"]), vec!["web"]);
    }

    #[test]
    fn select_repositories_rejects_the_first_unknown_name() {
        let names = ["api".to_string(), "ghost".to_string(), "other".to_string()];
        let error = spex()
            .select_repositories(&names)
            .expect_err("unknown repo");
        assert!(matches!(
            error.current_context(),
            CliError::UnknownRepo(name) if name == "ghost"
        ));
    }
}
