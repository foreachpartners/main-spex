use crate::config::{Config, Repository};
use crate::rule::Rule;
use crate::spec::{AcceptanceCriterion, ContractLevel, Spec, Statement, Status};

pub fn spec(id: &str) -> Spec {
    Spec {
        id: id.to_string(),
        path: format!("{id}.yaml"),
        description: String::new(),
        acceptance_criteria: Vec::new(),
        blocked_by: Vec::new(),
        scope: Vec::new(),
        status: Status::Planned,
        contract: ContractLevel::Single,
        coverage_ignore: Vec::new(),
    }
}

pub fn rule(id: &str) -> Rule {
    Rule {
        id: id.to_string(),
        path: format!("{id}.yaml"),
        acceptance_criteria: Vec::new(),
        checks: Vec::new(),
    }
}

pub fn criterion(id: &str) -> AcceptanceCriterion {
    AcceptanceCriterion {
        id: id.to_string(),
        statement: Statement::Must(id.to_string()),
        is_done: None,
        line: 0,
    }
}

pub fn repository(name: &str, path: &str) -> Repository {
    Repository {
        repository_name: name.to_string(),
        repository_short_name: name.to_string(),
        description: None,
        path: path.to_string(),
        contracts: Vec::new(),
        rules: Vec::new(),
        ignored_errors: Vec::new(),
    }
}

pub fn config(repositories: Vec<Repository>) -> Config {
    Config {
        project_name: "demo".to_string(),
        cli_version: "0.0.0".to_string(),
        description: None,
        rules_unused: Vec::new(),
        repositories,
    }
}
