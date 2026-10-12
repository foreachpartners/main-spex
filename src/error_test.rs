#[cfg(test)]
mod tests {
    use eros::Context;

    use super::super::*;

    #[test]
    fn describe_joins_contexts_from_outermost_to_root_cause() {
        let error = Err::<(), _>(eros::error!("disk full"))
            .context("could not write the report")
            .context("could not save")
            .unwrap_err();

        assert_eq!(
            describe(&error),
            "could not save: could not write the report: disk full"
        );
    }

    #[test]
    fn describe_names_only_the_root_cause_when_there_is_no_context() {
        assert_eq!(describe(&eros::error!("disk full")), "disk full");
    }

    #[test]
    fn describe_omits_source_locations() {
        let error = config_not_found().context("could not load");

        assert!(!describe(&error).contains("src/"));
    }

    #[test]
    fn config_not_found_names_the_config_file() {
        assert!(describe(&config_not_found()).starts_with("`spex.yaml` not found"));
    }
}
