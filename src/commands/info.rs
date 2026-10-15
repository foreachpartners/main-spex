use std::process::ExitCode;

use clap::Args;
use eros::{Context, IntoAnyUnion};
use serde::Serialize;
use serde_json::Value;

use crate::config::Package;
use crate::config::spex::Spex;
use crate::error::Result;
use crate::support::fs::canonical;

#[derive(Args)]
pub struct InfoArgs {
    #[arg(
        long,
        num_args = 1..,
        help = "Filter the packages list (by name or short_name)"
    )]
    package: Vec<String>,
}

#[derive(Serialize)]
struct Info {
    spex_dir: String,
    #[serde(flatten)]
    config: Value,
}

pub fn run(args: InfoArgs) -> Result<ExitCode> {
    let (dir, config) = Spex::discover_config()?;
    let mut config_json = to_json(&config)?;
    let raw_packages = config
        .packages
        .iter()
        .map(to_json)
        .collect::<Result<Vec<_>>>()?;
    let spex = Spex::from_config(dir, config).with_package_configs()?;
    let selected = spex.select_packages(&args.package)?;

    config_json["packages"] = Value::Array(
        raw_packages
            .into_iter()
            .zip(&spex.packages)
            .filter(|(_, package)| {
                selected
                    .iter()
                    .any(|picked| std::ptr::eq(*picked, *package))
            })
            .map(|(json, package)| merged_package(json, package))
            .collect::<Result<_>>()?,
    );
    config_json["coverage_ignore"] = to_json(&spex.coverage_ignore)?;
    config_json["agent_root"] =
        Value::String(canonical(&spex.agent_root).to_string_lossy().into_owned());
    let info = Info {
        spex_dir: spex.dir.to_string_lossy().into_owned(),
        config: config_json,
    };
    let json = serde_json::to_string_pretty(&info)
        .context("could not serialize the spex configuration")?;
    println!("{json}");

    Ok(ExitCode::SUCCESS)
}

fn to_json(value: &impl Serialize) -> Result<Value> {
    serde_json::to_value(value)
        .context("could not serialize the spex configuration")
        .any_union()
}

fn merged_package(mut json: Value, package: &Package) -> Result<Value> {
    json["path"] = Value::String(canonical(&package.path).to_string_lossy().into_owned());
    json["ignored_errors"] = to_json(&package.ignored_errors)?;
    Ok(json)
}
