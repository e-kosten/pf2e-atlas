use icu_properties::{
    CodePointMapData, CodePointSetData,
    props::{Alphabetic, GeneralCategory, GeneralCategoryGroup, JoinControl},
};

fn word_character(ch: char) -> bool {
    let category = CodePointMapData::<GeneralCategory>::new().get(ch);
    CodePointSetData::new::<Alphabetic>().contains(ch)
        || GeneralCategoryGroup::Mark.contains(category)
        || category == GeneralCategory::DecimalNumber
        || CodePointSetData::new::<JoinControl>().contains(ch)
}

/// Foundry PF2e's default sluggify mode, used by NPC attack effects and pack builds.
pub fn foundry_slug(text: &str) -> String {
    if text == "-" {
        return text.into();
    }
    let categories = CodePointMapData::<GeneralCategory>::new();
    let mut split_camel = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut previous = None;
    while let Some(ch) = chars.next() {
        let category = categories.get(ch);
        if previous == Some(GeneralCategory::LowercaseLetter)
            && category == GeneralCategory::UppercaseLetter
            && chars.peek().is_none_or(|next| word_character(*next))
        {
            split_camel.push('-');
        }
        split_camel.push(ch);
        previous = Some(category);
    }
    // Lowercase the entire string for context-sensitive Unicode mappings.
    let mut slug = String::with_capacity(split_camel.len());
    let mut separator = false;
    for ch in split_camel.to_lowercase().chars() {
        match ch {
            '\'' | '’' => {}
            ch if word_character(ch) => {
                if separator {
                    slug.push('-');
                    separator = false;
                }
                slug.push(ch);
            }
            _ => separator = !slug.is_empty(),
        }
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::foundry_slug;

    #[test]
    fn foundry_slug_matches_pinned_default_mode() {
        for (input, expected) in [
            (" Vile Innards ", "vile-innards"),
            ("-", "-"),
            (" --- ", ""),
            ("Witch’s (Witch's) Eye", "witchs-witchs-eye"),
            ("fooBar, fooX!", "foo-bar-foox"),
            ("aBcDe", "a-bc-de"),
            ("Élan e\u{301}lan", "élan-e\u{301}lan"),
            ("Step ² ٢", "step-٢"),
            ("ΟΣ", "ος"),
            ("A\u{200d}B", "a\u{200d}b"),
        ] {
            assert_eq!(foundry_slug(input), expected, "{input:?}");
        }
    }
}
