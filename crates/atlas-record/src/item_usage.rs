/// Derives the established filter vocabulary from a typed Foundry item usage slug.
///
/// Only the admitted exact slugs establish a hands requirement. This is a query
/// projection only; occurrence-local equipped state never participates.
pub fn hands_requirement_from_usage(usage: &str) -> Option<&'static str> {
    match usage {
        "held-in-two-hands" => Some("two_hands"),
        "held-in-one-plus-hands" => Some("one_plus_hands"),
        "held-in-one-hand" => Some("one_hand"),
        _ => None,
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
        for usage in [
            "not-held-in-one-hand",
            "held-in-one-hand-sideways",
            "held-in-two-hands-invalid",
            "held-in-one-plus-hands-invalid",
            " held-in-one-hand",
            "held-in-one-hand ",
            "",
        ] {
            assert_eq!(hands_requirement_from_usage(usage), None, "{usage}");
        }
    }
}
