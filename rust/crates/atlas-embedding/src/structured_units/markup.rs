#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum MarkupBlock {
    Heading {
        level: u8,
        label: String,
    },
    Paragraph {
        text: String,
        first_label: Option<String>,
    },
    List(Vec<ListItem>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ListItem {
    pub(super) label: Option<String>,
    pub(super) body: String,
}

pub(super) fn markup_blocks(markup: &str) -> Vec<MarkupBlock> {
    let tokens = tokenize_markup(markup);
    let mut blocks = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        match tokens[index].as_str() {
            tag if heading_level(tag).is_some() && !tag.starts_with("</") => {
                let level = heading_level(tag).expect("checked heading level");
                let (inner, next) = collect_until_closing(&tokens, index + 1, &format!("h{level}"));
                let label = strip_markup_for_embedding_units(&inner.join(""));
                if !label.trim().is_empty() {
                    blocks.push(MarkupBlock::Heading { level, label });
                }
                index = next;
            }
            tag if tag_open_name(tag) == Some("p") => {
                let (inner, next) = collect_until_closing(&tokens, index + 1, "p");
                let html = inner.join("");
                let text = strip_markup_for_embedding_units(&html);
                if !text.trim().is_empty() {
                    blocks.push(MarkupBlock::Paragraph {
                        first_label: first_strong_label(&html),
                        text,
                    });
                }
                index = next;
            }
            tag if matches!(tag_open_name(tag), Some("ul" | "ol")) => {
                let (inner, next) =
                    collect_until_closing(&tokens, index + 1, tag_open_name(tag).unwrap());
                let items = list_items_from_tokens(&inner);
                if !items.is_empty() {
                    blocks.push(MarkupBlock::List(items));
                }
                index = next;
            }
            tag if tag_open_name(tag) == Some("table") => {
                let (inner, next) = collect_until_closing(&tokens, index + 1, "table");
                let table_text = render_table_for_embedding(&inner.join(""));
                if !table_text.is_empty() {
                    blocks.push(MarkupBlock::Paragraph {
                        text: table_text,
                        first_label: None,
                    });
                }
                index = next;
            }
            token if token.starts_with('<') => {
                index += 1;
            }
            _ => {
                let mut text = String::new();
                while index < tokens.len() && !tokens[index].starts_with('<') {
                    text.push_str(&tokens[index]);
                    index += 1;
                }
                let text = strip_markup_for_embedding_units(&text);
                if !text.trim().is_empty() {
                    blocks.push(MarkupBlock::Paragraph {
                        text,
                        first_label: None,
                    });
                }
            }
        }
    }
    blocks
}

fn list_items_from_tokens(tokens: &[String]) -> Vec<ListItem> {
    let mut items = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        match tag_open_name(&tokens[index]) {
            Some("li") => {
                let (inner, next) = collect_until_closing(tokens, index + 1, "li");
                let html = inner.join("");
                let label = first_strong_label(&html);
                let body = strip_markup_for_embedding_units(&html);
                if !body.trim().is_empty() {
                    items.push(ListItem { label, body });
                }
                index = next;
            }
            Some("ul" | "ol") => {
                let (_, next) = collect_until_closing(
                    tokens,
                    index + 1,
                    tag_open_name(&tokens[index]).unwrap(),
                );
                index = next;
            }
            _ => {
                index += 1;
            }
        }
    }
    items
}

pub(super) fn render_blocks_for_unit(blocks: &[MarkupBlock]) -> String {
    blocks
        .iter()
        .map(|block| match block {
            MarkupBlock::Heading { label, .. } => label.clone(),
            MarkupBlock::Paragraph { text, .. } => text.clone(),
            MarkupBlock::List(items) => items
                .iter()
                .map(|item| item.body.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        })
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn strip_markup_for_embedding_units(markup: &str) -> String {
    strip_markup(&drop_table_body_cells(markup))
}

fn drop_table_body_cells(markup: &str) -> String {
    let tokens = tokenize_markup(markup);
    let mut output = String::new();
    let mut index = 0;
    while index < tokens.len() {
        if tag_open_name(&tokens[index]) == Some("td") {
            let (_, next) = collect_until_closing(&tokens, index + 1, "td");
            index = next;
            continue;
        }
        output.push_str(&tokens[index]);
        index += 1;
    }
    output
}

fn render_table_for_embedding(table_markup: &str) -> String {
    let tokens = tokenize_markup(table_markup);
    let mut captions = Vec::new();
    let mut headers = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        match tag_open_name(&tokens[index]) {
            Some("caption") => {
                let (inner, next) = collect_until_closing(&tokens, index + 1, "caption");
                let text = strip_markup_for_embedding_units(&inner.join(""));
                if !text.is_empty() {
                    captions.push(text);
                }
                index = next;
            }
            Some("th") => {
                let (inner, next) = collect_until_closing(&tokens, index + 1, "th");
                let text = strip_markup_for_embedding_units(&inner.join(""));
                if !text.is_empty() {
                    headers.push(text);
                }
                index = next;
            }
            _ => index += 1,
        }
    }
    let mut parts = Vec::new();
    if !captions.is_empty() {
        parts.push(format!("Table: {}", captions.join(" | ")));
    }
    if !headers.is_empty() {
        parts.push(format!("Columns: {}", headers.join(" | ")));
    }
    parts.join(" ")
}

fn first_strong_label(html: &str) -> Option<String> {
    let tokens = tokenize_markup(html);
    let mut index = 0;
    while index < tokens.len() {
        if tag_open_name(&tokens[index]) == Some("strong") {
            let (inner, _) = collect_until_closing(&tokens, index + 1, "strong");
            let label = strip_markup_for_embedding_units(&inner.join(""));
            return (!label.trim().is_empty()).then_some(label);
        }
        if tokens[index].starts_with('<') || tokens[index].trim().is_empty() {
            index += 1;
            continue;
        }
        break;
    }
    None
}

fn tokenize_markup(markup: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut offset = 0;
    while offset < markup.len() {
        let rest = &markup[offset..];
        if rest.starts_with('<')
            && let Some(end) = rest.find('>')
        {
            tokens.push(rest[..=end].to_string());
            offset += end + 1;
            continue;
        }
        let next_tag = rest.find('<').unwrap_or(rest.len());
        if next_tag > 0 {
            tokens.push(rest[..next_tag].to_string());
            offset += next_tag;
        } else {
            offset += 1;
        }
    }
    tokens
}

fn collect_until_closing(tokens: &[String], start: usize, tag_name: &str) -> (Vec<String>, usize) {
    let mut depth = 1usize;
    let mut inner = Vec::new();
    let mut index = start;
    while index < tokens.len() {
        let token = &tokens[index];
        if tag_open_name(token) == Some(tag_name) {
            depth += 1;
        } else if tag_close_name(token) == Some(tag_name) {
            depth -= 1;
            if depth == 0 {
                return (inner, index + 1);
            }
        }
        inner.push(token.clone());
        index += 1;
    }
    (inner, index)
}

fn heading_level(tag: &str) -> Option<u8> {
    let name = tag_open_name(tag)?;
    let suffix = name.strip_prefix('h')?;
    let level = suffix.parse::<u8>().ok()?;
    (1..=6).contains(&level).then_some(level)
}

fn tag_open_name(tag: &str) -> Option<&str> {
    if !tag.starts_with('<') || tag.starts_with("</") || tag.starts_with("<!") {
        return None;
    }
    tag[1..]
        .split(|character: char| character.is_ascii_whitespace() || character == '>')
        .next()
        .map(|name| name.trim_end_matches('/'))
        .filter(|name| !name.is_empty())
}

fn tag_close_name(tag: &str) -> Option<&str> {
    if !tag.starts_with("</") {
        return None;
    }
    tag[2..]
        .split(|character: char| character.is_ascii_whitespace() || character == '>')
        .next()
        .filter(|name| !name.is_empty())
}

pub(super) fn normalize_split_label(value: &str) -> String {
    strip_markup_for_embedding_units(value)
        .trim_matches(|character: char| {
            character.is_ascii_punctuation() || character.is_whitespace()
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub(super) fn normalized_token_count(value: &str) -> usize {
    normalize_split_label(value).split_whitespace().count()
}

fn strip_markup(value: &str) -> String {
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

fn parse_foundry_inline(value: &str, start: usize) -> Option<(String, usize)> {
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
            return Some((strip_markup(&value[display_start..display_end]), end));
        }
    }

    Some((foundry_macro_signal(name, body), end))
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
