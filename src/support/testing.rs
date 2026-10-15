use std::path::Path;

use crate::config::spex::Spex;
use crate::config::{Layout, Package, SpexConfig};
use crate::rule::{RULES_DIR, Rule, RuleId};
use crate::spec::{
    AcId, AcceptanceCriterion, ContractLevel, SPECS_DIR, Spec, SpecId, Statement, Status,
};

pub fn spex_dir() -> &'static Path {
    Path::new("/spex")
}

pub fn spec_id(id: &str) -> SpecId {
    id.to_string().try_into().expect("valid spec id")
}

pub fn ac_id(id: &str) -> AcId {
    id.to_string()
        .try_into()
        .expect("valid acceptance criterion id")
}

pub fn rule_id(id: &str) -> RuleId {
    id.to_string().try_into().expect("valid rule id")
}

pub fn spec(id: &str) -> Spec {
    Spec {
        id: spec_id(id),
        path: spex_dir().join(SPECS_DIR).join(format!("{id}.yaml")),
        description: String::new(),
        acceptance_criteria: Vec::new(),
        blocked_by: Vec::new(),
        packages: Vec::new(),
        status: Status::Planned,
        contract: ContractLevel::Single,
        coverage_ignore: Vec::new(),
    }
}

pub fn rule(id: &str) -> Rule {
    Rule {
        id: rule_id(id),
        path: spex_dir().join(RULES_DIR).join(format!("{id}.yaml")),
        acceptance_criteria: Vec::new(),
        checks: Vec::new(),
    }
}

pub fn criterion(id: &str) -> AcceptanceCriterion {
    AcceptanceCriterion {
        id: ac_id(id),
        statement: Statement::Must(id.to_string()),
        is_done: true,
        line: 0,
    }
}

pub fn package(name: &str, path: &str) -> Package {
    Package {
        package_name: name.to_string(),
        package_short_name: name.to_string(),
        path: spex_dir().join(path),
        contracts: Vec::new(),
        rules: Vec::new(),
        ignored_errors: Vec::new(),
        is_spex_dir: false,
    }
}

pub fn spex(packages: Vec<Package>) -> Spex {
    Spex {
        dir: spex_dir().to_path_buf(),
        project_name: "demo".to_string(),
        agent_root: spex_dir().to_path_buf(),
        packages,
        coverage_ignore: Vec::new(),
    }
}

pub fn spex_config() -> SpexConfig {
    SpexConfig {
        project_name: "demo".to_string(),
        cli_version: "0.0.0".to_string(),
        layout: Layout::MultiRepo,
        agent_root: ".".to_string(),
        description: None,
        rules_unused: Vec::new(),
        packages: Vec::new(),
        coverage_ignore: Vec::new(),
    }
}
