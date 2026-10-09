#[cfg(test)]
mod tests {
    use error_stack::ResultExt;

    use super::super::*;

    #[test]
    fn describe_joins_contexts_from_outermost_to_root_cause() {
        let report = Err::<(), _>(Report::new(CliError::Custom("disk full".into())))
            .change_context(CliError::Custom("could not save".into()))
            .unwrap_err();

        assert_eq!(describe(&report), "could not save: disk full");
    }

    #[test]
    fn describe_omits_source_locations() {
        let report = Report::new(CliError::ConfigNotFound);

        assert!(!describe(&report).contains("src/"));
    }
}
