/// Derives the established filter vocabulary from a typed Foundry item usage slug.
///
/// The order is significant because the longer forms contain the one-hand
/// spelling. This is a query projection only; occurrence-local equipped state
/// never participates.
pub fn hands_requirement_from_usage(usage: &str) -> Option<&'static str> {
    if usage.contains("held-in-two-hands") {
        Some("two_hands")
    } else if usage.contains("held-in-one-plus-hands") {
        Some("one_plus_hands")
    } else if usage.contains("held-in-one-hand") {
        Some("one_hand")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::hands_requirement_from_usage;

    #[test]
    fn derives_only_the_existing_hands_filter_vocabulary() {
        assert_eq!(
            hands_requirement_from_usage("held-in-one-hand"),
            Some("one_hand")
        );
        assert_eq!(
            hands_requirement_from_usage("held-in-one-plus-hands"),
            Some("one_plus_hands")
        );
        assert_eq!(
            hands_requirement_from_usage("held-in-two-hands"),
            Some("two_hands")
        );
        assert_eq!(hands_requirement_from_usage("worn"), None);
    }
}
