#[cfg(test)]
mod tests {
    use super::super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn order_runs_from_must_to_may() {
        assert_eq!(
            Commitment::order(),
            "MUST, MUST NOT, SHOULD, SHOULD NOT, MAY"
        );
    }

    #[test]
    fn commitments_compare_in_that_order() {
        assert!(Commitment::iter().is_sorted());
    }

    #[test]
    fn from_text_reads_the_leading_commitment() {
        for commitment in Commitment::iter() {
            let text = format!("{} do something", commitment.label());
            assert_eq!(Commitment::from_text(&text), Some(commitment));
        }
    }

    #[test]
    fn from_text_prefers_the_longer_commitment() {
        assert_eq!(
            Commitment::from_text("MUST NOT leak"),
            Some(Commitment::MustNot)
        );
        assert_eq!(
            Commitment::from_text("SHOULD NOT leak"),
            Some(Commitment::ShouldNot)
        );
    }

    #[test]
    fn from_text_is_none_without_a_commitment() {
        assert_eq!(Commitment::from_text("plain statement"), None);
    }
}
