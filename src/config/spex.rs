use std::path::PathBuf;

use anstream::eprintln;
use eros::bail;
use owo_colors::OwoColorize;

use crate::config::{RepoConfig, Repository, SPEX_CONFIG_FILE, SpexConfig};
use crate::error::Result;
use crate::rule::{self, Rule};
use crate::spec::{self, Spec, SpecCoverageIgnore};
use crate::support::fs::current_dir;

#[cfg(test)]
#[path = "spex_test.rs"]
mod spex_test;

pub struct Spex {
    pub dir: PathBuf,
    pub project_name: String,
    pub repositories: Vec<Repository>,
    pub coverage_ignore: Vec<SpecCoverageIgnore>,
}

impl Spex {
    pub fn locate_spex_config() -> Result<PathBuf> {
        SpexConfig::find_file(&current_dir()?)
    }

    pub fn discover() -> Result<Self> {
        let (dir, config) = Self::discover_config()?;
        Self::from_config(dir, config).with_repo_configs()
    }

    pub fn discover_config() -> Result<(PathBuf, SpexConfig)> {
        let config_path = Self::locate_spex_config()?;
        let config = SpexConfig::load(&config_path)?;
        warn_if_cli_outdated(&config);
        Ok((SpexConfig::dir_of(&config_path).to_path_buf(), config))
    }

    pub fn from_config(dir: PathBuf, config: SpexConfig) -> Self {
        let repositories = config
            .repositories
            .into_iter()
            .map(|repo| Repository::from_config(repo, &dir))
            .collect();
        Self {
            dir,
            project_name: config.project_name,
            repositories,
            coverage_ignore: config.coverage_ignore,
        }
    }

    fn with_repo_configs(mut self) -> Result<Self> {
        for repo in &mut self.repositories {
            let repo_config = RepoConfig::load(&repo.path)?;
            self.coverage_ignore.extend(repo_config.coverage_ignore);
            repo.ignored_errors.extend(repo_config.ignored_errors);
        }
        Ok(self)
    }

    pub fn rules(&self) -> Result<Vec<Rule>> {
        Rule::load_all(&self.dir.join(rule::RULES_DIR))
    }

    pub fn specs(&self) -> Result<Vec<Spec>> {
        let mut specs = Spec::load_all(&self.dir.join(spec::SPECS_DIR))?;
        for spec in &mut specs {
            let waivers = self
                .coverage_ignore
                .iter()
                .filter(|waiver| waiver.spec_id == spec.id)
                .map(|waiver| waiver.ignore.clone());
            spec.coverage_ignore.extend(waivers);
        }
        Ok(specs)
    }

    pub fn validate_repository_names(&self, names: &[String]) -> Result<()> {
        for name in names {
            if !self
                .repositories
                .iter()
                .any(|repo| repo.matches_any_repo_name(name))
            {
                bail!("unknown workspace repository `{name}` — not listed in {SPEX_CONFIG_FILE}");
            }
        }
        Ok(())
    }

    pub fn select_repositories(&self, names: &[String]) -> Result<Vec<&Repository>> {
        self.validate_repository_names(names)?;
        Ok(self
            .repositories
            .iter()
            .filter(|repo| {
                names.is_empty() || names.iter().any(|name| repo.matches_any_repo_name(name))
            })
            .collect())
    }

    pub fn validate_scopes(&self, scopes: &[String]) -> Result<()> {
        for short_name in scopes {
            if !self
                .repositories
                .iter()
                .any(|repo| &repo.repository_short_name == short_name)
            {
                bail!(
                    "unknown repository short name `{short_name}` — not listed in {SPEX_CONFIG_FILE}"
                );
            }
        }
        Ok(())
    }
}

fn warn_if_cli_outdated(config: &SpexConfig) {
    if let Some(warning) = config.version_warning() {
        eprintln!("{} {warning}", "warning:".yellow());
    }
}
