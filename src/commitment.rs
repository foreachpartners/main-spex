#[cfg(test)]
#[path = "commitment_test.rs"]
mod commitment_test;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Commitment {
    Must,
    MustNot,
    Should,
    ShouldNot,
    May,
}

impl Commitment {
    pub const ALL: [Commitment; 5] = [
        Commitment::Must,
        Commitment::MustNot,
        Commitment::Should,
        Commitment::ShouldNot,
        Commitment::May,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Commitment::Must => "MUST",
            Commitment::MustNot => "MUST NOT",
            Commitment::Should => "SHOULD",
            Commitment::ShouldNot => "SHOULD NOT",
            Commitment::May => "MAY",
        }
    }

    pub fn order() -> String {
        Self::ALL.map(Self::label).join(", ")
    }

    pub fn from_text(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .filter(|commitment| text.starts_with(commitment.label()))
            .max_by_key(|commitment| commitment.label().len())
    }
}
