pub(crate) fn normalize_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub(crate) fn strip_markup(value: &str) -> String {
    let mut output = String::new();
    let mut offset = 0;

    while offset < value.len() {
        let rest = &value[offset..];
        if rest.starts_with('<')
            && let Some(close) = rest.find('>')
        {
            output.push(' ');
            offset += close + 1;
            continue;
        }

        if rest.starts_with('@')
            && let Some((replacement, end)) = parse_foundry_inline(value, offset)
        {
            if !replacement.is_empty() {
                output.push(' ');
                output.push_str(&replacement);
                output.push(' ');
            } else {
                output.push(' ');
            }
            offset = end;
            continue;
        }

        let character = rest
            .chars()
            .next()
            .expect("strip_markup offset should be inside the input");
        match character {
            '>' => output.push(' '),
            _ => output.push(character),
        }
        offset += character.len_utf8();
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn dropped_foundry_inline_macros(value: &str) -> Vec<DroppedFoundryInlineMacro> {
    let mut dropped = Vec::new();
    let mut offset = 0;

    while offset < value.len() {
        let rest = &value[offset..];
        if rest.starts_with('@')
            && let Some(parsed) = parse_foundry_inline_macro(value, offset)
        {
            if parsed.replacement.is_empty() {
                dropped.push(DroppedFoundryInlineMacro {
                    name: parsed.name,
                    raw: value[offset..parsed.end].to_string(),
                });
            }
            offset = parsed.end;
            continue;
        }

        let Some(character) = rest.chars().next() else {
            break;
        };
        offset += character.len_utf8();
    }

    dropped
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DroppedFoundryInlineMacro {
    pub(crate) name: String,
    pub(crate) raw: String,
}

fn parse_foundry_inline(value: &str, start: usize) -> Option<(String, usize)> {
    parse_foundry_inline_macro(value, start).map(|parsed| (parsed.replacement, parsed.end))
}

fn parse_foundry_inline_macro(value: &str, start: usize) -> Option<ParsedFoundryInlineMacro> {
    let rest = &value[start..];
    if !rest.starts_with('@') {
        return None;
    }

    let name_start = start + 1;
    let mut name_end = name_start;
    for (relative, character) in value[name_start..].char_indices() {
        if character.is_ascii_alphabetic() {
            name_end = name_start + relative + character.len_utf8();
        } else {
            break;
        }
    }
    if name_end == name_start || !value[name_end..].starts_with('[') {
        return None;
    }
    let name = &value[name_start..name_end];

    let body_start = name_end + 1;
    let body_end = balanced_close(value, body_start, '[', ']')?;
    let body = &value[body_start..body_end];
    let mut end = body_end + 1;

    if value[end..].starts_with('{') {
        let display_start = end + 1;
        if let Some(display_end) = balanced_close(value, display_start, '{', '}') {
            end = display_end + 1;
            return Some(ParsedFoundryInlineMacro {
                name: name.to_string(),
                replacement: strip_markup(&value[display_start..display_end]),
                end,
            });
        }
    }

    Some(ParsedFoundryInlineMacro {
        name: name.to_string(),
        replacement: foundry_macro_signal(name, body),
        end,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedFoundryInlineMacro {
    name: String,
    replacement: String,
    end: usize,
}

fn balanced_close(value: &str, body_start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 1;
    for (relative, character) in value[body_start..].char_indices() {
        match character {
            current if current == open => depth += 1,
            current if current == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(body_start + relative);
                }
            }
            _ => {}
        }
    }
    None
}

fn foundry_macro_signal(name: &str, body: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "damage" => nested_bracket_signals(body).join(" "),
        "check" => mechanic_terms(body.split('|').next().unwrap_or_default()).join(" "),
        "trait" => mechanic_terms(body).join(" "),
        _ => String::new(),
    }
}

fn nested_bracket_signals(value: &str) -> Vec<String> {
    let mut signals = Vec::new();
    let mut offset = 0;

    while offset < value.len() {
        let rest = &value[offset..];
        let Some(open_relative) = rest.find('[') else {
            break;
        };
        let body_start = offset + open_relative + 1;
        let Some(body_end) = balanced_close(value, body_start, '[', ']') else {
            break;
        };
        signals.extend(mechanic_terms(&value[body_start..body_end]));
        offset = body_end + 1;
    }

    signals
}

fn mechanic_terms(value: &str) -> Vec<String> {
    value
        .split(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    '|' | ':' | ';' | ',' | '=' | '[' | ']' | '(' | ')' | '{' | '}'
                )
        })
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .filter(|term| !term.chars().all(|character| character.is_ascii_digit()))
        .filter(|term| !term.contains(char::is_numeric))
        .filter(|term| {
            !matches!(
                term.to_ascii_lowercase().as_str(),
                "dc" | "basic" | "save" | "saving" | "throw" | "damage"
            )
        })
        .map(ToOwned::to_owned)
        .collect()
}

pub(crate) fn create_search_text(
    name: &str,
    description: Option<&str>,
    traits: &[String],
) -> String {
    [Some(name), description, Some(&traits.join(" "))]
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::strip_markup;

    #[test]
    fn strip_markup_keeps_uuid_display_text_without_foundry_path() {
        let text = strip_markup(
            "<p><strong>Stage 1</strong> \
             @UUID[Compendium.pf2e.conditionitems.Item.Sickened]{Sickened 1} (1 day)</p>",
        );

        assert_eq!(text, "Stage 1 Sickened 1 (1 day)");
    }

    #[test]
    fn strip_markup_keeps_clear_damage_and_check_signals() {
        let text = strip_markup(
            "<p><strong>Saving Throw</strong> @Check[fortitude|dc:21]</p>\
             <p><strong>Stage 1</strong> @Damage[1d8[poison]] damage</p>",
        );

        assert_eq!(text, "Saving Throw fortitude Stage 1 poison damage");
    }

    #[test]
    fn strip_markup_drops_reference_without_display_text() {
        let text =
            strip_markup("See @UUID[Compendium.pf2e.conditionitems.Item.Sickened] for details.");

        assert_eq!(text, "See for details.");
    }

    #[test]
    fn dropped_foundry_inline_macros_reports_empty_replacements() {
        let dropped = super::dropped_foundry_inline_macros(
            "@UUID[Compendium.pf2e.conditionitems.Item.Sickened] \
             @Template[type:burst|distance:10] \
             @UUID[Compendium.pf2e.conditionitems.Item.Clumsy]{Clumsy 1}",
        );

        assert_eq!(
            dropped
                .iter()
                .map(|macro_drop| (macro_drop.name.as_str(), macro_drop.raw.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (
                    "UUID",
                    "@UUID[Compendium.pf2e.conditionitems.Item.Sickened]"
                ),
                ("Template", "@Template[type:burst|distance:10]")
            ]
        );
    }
}
