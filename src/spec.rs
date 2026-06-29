use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    #[default]
    Planned,
    Blocked,
    Deprecated,
}

impl Status {
    pub const ALL: [Status; 4] = [
        Status::Active,
        Status::Planned,
        Status::Blocked,
        Status::Deprecated,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Status::Active => "active",
            Status::Planned => "planned",
            Status::Blocked => "blocked",
            Status::Deprecated => "deprecated",
        }
    }

    pub fn is_implemented(self) -> bool {
        matches!(self, Status::Active | Status::Deprecated)
    }
}

fn ac_implemented(criterion: &str) -> bool {
    criterion.trim_start().starts_with("[x]")
}

pub fn ac_counts(criteria: &[String]) -> (usize, usize) {
    let done = criteria
        .iter()
        .filter(|criterion| ac_implemented(criterion))
        .count();
    (done, criteria.len())
}
