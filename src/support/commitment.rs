use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

#[cfg(test)]
#[path = "commitment_test.rs"]
mod commitment_test;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Commitment {
    Must,
    #[strum(serialize = "MUST NOT")]
    MustNot,
    Should,
    #[strum(serialize = "SHOULD NOT")]
    ShouldNot,
    May,
}

impl Commitment {
    pub fn label(self) -> &'static str {
        self.into()
    }

    pub fn order() -> String {
        Self::iter().map(Self::label).collect::<Vec<_>>().join(", ")
    }

    pub fn from_text(text: &str) -> Option<Self> {
        Self::iter()
            .filter(|commitment| text.starts_with(commitment.label()))
            .max_by_key(|commitment| commitment.label().len())
    }
}
