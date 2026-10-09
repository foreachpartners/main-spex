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
}
