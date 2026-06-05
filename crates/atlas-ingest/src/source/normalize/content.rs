use std::collections::{BTreeMap, BTreeSet};

use ego_tree::NodeRef;
use scraper::{Html, node::Node};

use atlas_record::{
    DamagePart, FoundryLink, FoundryLinkBehavior, FoundryLinkMacroKind, FoundryLinkSource,
    FoundryNode, RichDocument, RichLinkTarget, RichNode, render_plain_text,
};

use super::content_diagnostics::ContentParseDiagnostics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedContentDocument {
    pub(crate) document: RichDocument,
    pub(crate) diagnostics: ContentParseDiagnostics,
}

pub(crate) fn parse_foundry_content(value: &str) -> ParsedContentDocument {
    let fragment = Html::parse_fragment(value);
    let mut state = ParseState::default();
    let mut nodes = Vec::new();
    for child in fragment.tree.root().children() {
        nodes.extend(convert_node_ref(child, &mut state));
    }

    let mut diagnostics = ContentParseDiagnostics::default();
    for tag in state.unsupported_tags {
        diagnostics.record_unsupported_tag(&tag);
    }

    ParsedContentDocument {
        document: RichDocument::new(nodes),
        diagnostics,
    }
}

#[derive(Default)]
struct ParseState {
    unsupported_tags: BTreeSet<String>,
}

fn convert_node_ref(node_ref: NodeRef<'_, Node>, state: &mut ParseState) -> Vec<RichNode> {
    match node_ref.value() {
        Node::Text(text) => parse_text_nodes(text, state),
        Node::Element(element) => {
            let tag = element.name().to_ascii_lowercase();
            if tag == "html" && element.attrs().next().is_none() {
                let mut children = Vec::new();
                for child in node_ref.children() {
                    children.extend(convert_node_ref(child, state));
                }
                return children;
            }
            if is_unusual_tag(&tag) {
                state.unsupported_tags.insert(tag.clone());
            }
            let attributes = element
                .attrs()
                .map(|(name, value)| (name.to_ascii_lowercase(), Some(value.to_string())))
                .collect::<BTreeMap<_, _>>();
            let mut children = Vec::new();
            for child in node_ref.children() {
                children.extend(convert_node_ref(child, state));
            }
            vec![RichNode::HtmlElement {
                tag,
                attributes,
                children,
            }]
        }
        Node::Comment(_) | Node::Doctype(_) => Vec::new(),
        _ => Vec::new(),
    }
}

fn is_unusual_tag(tag: &str) -> bool {
    !matches!(
        tag,
        "a" | "article"
            | "b"
            | "blockquote"
            | "body"
            | "br"
            | "caption"
            | "code"
            | "dd"
            | "div"
            | "dl"
            | "dt"
            | "em"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "hr"
            | "i"
            | "img"
            | "li"
            | "main"
            | "ol"
            | "p"
            | "section"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "sup"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
            | "ul"
    )
}

fn parse_text_nodes(value: &str, state: &mut ParseState) -> Vec<RichNode> {
    let mut nodes = Vec::new();
    let mut offset = 0;
    while offset < value.len() {
        let rest = &value[offset..];
        if rest.starts_with("[[/")
            && let Some(parsed) = parse_inline_command(value, offset, state)
        {
            nodes.push(RichNode::Foundry { node: parsed.node });
            offset = parsed.end;
            continue;
        }
        if rest.starts_with('@')
            && let Some(parsed) = parse_foundry_macro(value, offset, state)
        {
            nodes.push(parsed.node);
            offset = parsed.end;
            continue;
        }

        let next_macro = rest.find('@').unwrap_or(rest.len());
        let next_inline_command = rest.find("[[/").unwrap_or(rest.len());
        let next_signal = next_macro.min(next_inline_command);
        if next_signal > 0 {
            push_text(&mut nodes, &rest[..next_signal]);
            offset += next_signal;
        } else if let Some(character) = rest.chars().next() {
            push_text(&mut nodes, &character.to_string());
            offset += character.len_utf8();
        } else {
            break;
        }
    }
    merge_adjacent_text(nodes)
}

fn push_text(nodes: &mut Vec<RichNode>, text: &str) {
    if !text.is_empty() {
        nodes.push(RichNode::Text {
            text: text.to_string(),
        });
    }
}

struct ParsedMacro {
    node: RichNode,
    end: usize,
}

struct ParsedFoundryNode {
    node: FoundryNode,
    end: usize,
}

fn parse_inline_command(
    value: &str,
    start: usize,
    state: &mut ParseState,
) -> Option<ParsedFoundryNode> {
    let rest = &value[start..];
    let body_end_relative = rest.find("]]")?;
    let body_end = start + body_end_relative;
    let body = value[start + 3..body_end].trim();
    let mut end = body_end + 2;
    let label = parse_optional_label(value, &mut end, state)?;
    let (command, arguments) = body
        .split_once(char::is_whitespace)
        .map(|(command, arguments)| (command.trim_start_matches('/'), arguments.trim()))
        .unwrap_or_else(|| (body.trim_start_matches('/'), ""));

    Some(ParsedFoundryNode {
        node: FoundryNode::InlineCommand {
            command: command.to_string(),
            arguments: arguments.to_string(),
            options: inline_command_options(arguments),
            label,
        },
        end,
    })
}

fn parse_foundry_macro(value: &str, start: usize, state: &mut ParseState) -> Option<ParsedMacro> {
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
    let label = parse_optional_label(value, &mut end, state)?;
    let macro_name = name.to_ascii_lowercase();
    let parsed_body = ParsedMacroBody::parse(body);

    let node = match macro_name.as_str() {
        "uuid" | "compendium" => {
            let target = parsed_body.first_positional().unwrap_or(body).to_string();
            RichNode::FoundryLink {
                link: unresolved_link(
                    if macro_name == "uuid" {
                        FoundryLinkMacroKind::Uuid
                    } else {
                        FoundryLinkMacroKind::Compendium
                    },
                    target,
                    label,
                    FoundryLinkBehavior::Reference,
                ),
            }
        }
        "embed" => {
            let embed = ParsedEmbedBody::parse(body);
            let inline = embed
                .options
                .get("inline")
                .is_some_and(|value| value != "false");
            let hr = embed.options.get("hr").map(|value| value != "false");
            RichNode::FoundryLink {
                link: unresolved_link(
                    FoundryLinkMacroKind::Embed,
                    embed.target,
                    label,
                    FoundryLinkBehavior::Embed {
                        inline,
                        hr,
                        options: embed.options,
                    },
                ),
            }
        }
        "check" => RichNode::Foundry {
            node: FoundryNode::Check {
                statistic: parsed_body.first_positional().map(ToOwned::to_owned),
                options: parsed_body.options,
                label,
            },
        },
        "damage" => {
            let formula = parsed_body.first_positional().unwrap_or(body).to_string();
            RichNode::Foundry {
                node: FoundryNode::Damage {
                    damage_parts: damage_parts(&formula),
                    formula,
                    options: parsed_body.options,
                    label,
                },
            }
        }
        "template" => RichNode::Foundry {
            node: FoundryNode::Template {
                shape: parsed_body
                    .options
                    .get("type")
                    .cloned()
                    .or_else(|| parsed_body.first_positional().map(ToOwned::to_owned)),
                options: parsed_body.options,
                label,
            },
        },
        "action" => RichNode::Foundry {
            node: FoundryNode::ActionGlyph {
                action: label
                    .as_ref()
                    .map(|label| render_plain_text(&RichDocument::new(label.clone())))
                    .filter(|text| !text.trim().is_empty())
                    .unwrap_or_else(|| parsed_body.first_positional().unwrap_or(body).to_string()),
            },
        },
        "trait" => RichNode::Foundry {
            node: FoundryNode::Trait {
                traits: split_at_depth(body, ',')
                    .into_iter()
                    .map(str::trim)
                    .filter(|term| !term.is_empty())
                    .map(ToOwned::to_owned)
                    .collect(),
                label,
            },
        },
        "localize" => RichNode::Foundry {
            node: FoundryNode::Localize {
                key: body.to_string(),
                value: label,
            },
        },
        _ => RichNode::Foundry {
            node: FoundryNode::UnknownFoundry {
                name: macro_name,
                body: Some(body.to_string()),
                label,
                raw: value[start..end].to_string(),
            },
        },
    };

    Some(ParsedMacro { node, end })
}

fn parse_optional_label(
    value: &str,
    end: &mut usize,
    state: &mut ParseState,
) -> Option<Option<Vec<RichNode>>> {
    if value[*end..].starts_with('{') {
        let label_start = *end + 1;
        let label_end = balanced_close(value, label_start, '{', '}')?;
        *end = label_end + 1;
        Some(Some(parse_text_nodes(
            &value[label_start..label_end],
            state,
        )))
    } else {
        Some(None)
    }
}

fn unresolved_link(
    macro_kind: FoundryLinkMacroKind,
    target: String,
    label: Option<Vec<RichNode>>,
    behavior: FoundryLinkBehavior,
) -> FoundryLink {
    let fallback_label = reference_display_fallback(&target);
    FoundryLink {
        target: RichLinkTarget::Unresolved {
            target: target.clone(),
            fallback_label,
        },
        label,
        source: FoundryLinkSource {
            macro_kind,
            authored_target: target,
            relation: None,
        },
        behavior,
    }
}

struct ParsedMacroBody {
    positional: Vec<String>,
    options: BTreeMap<String, String>,
}

impl ParsedMacroBody {
    fn parse(body: &str) -> Self {
        let mut positional = Vec::new();
        let mut options = BTreeMap::new();

        for segment in split_at_depth(body, '|') {
            let segment = segment.trim();
            if segment.is_empty() {
                continue;
            }
            if let Some((key, value)) = split_option(segment) {
                options.insert(key.to_string(), value.to_string());
            } else if positional.is_empty() {
                positional.push(segment.to_string());
            } else {
                options.insert(segment.to_string(), "true".to_string());
            }
        }

        Self {
            positional,
            options,
        }
    }

    fn first_positional(&self) -> Option<&str> {
        self.positional.first().map(String::as_str)
    }
}

struct ParsedEmbedBody {
    target: String,
    options: BTreeMap<String, String>,
}

impl ParsedEmbedBody {
    fn parse(body: &str) -> Self {
        let mut segments = body.split_whitespace();
        let target = segments.next().unwrap_or(body).to_string();
        let mut options = BTreeMap::new();
        for segment in segments {
            if let Some((key, value)) = split_option(segment) {
                options.insert(key.to_string(), value.to_string());
            } else if !segment.is_empty() {
                options.insert(segment.to_string(), "true".to_string());
            }
        }
        Self { target, options }
    }
}

fn balanced_close(value: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 1;
    for (relative, character) in value[start..].char_indices() {
        if character == open {
            depth += 1;
        } else if character == close {
            depth -= 1;
            if depth == 0 {
                return Some(start + relative);
            }
        }
    }
    None
}

fn split_option(segment: &str) -> Option<(&str, &str)> {
    segment
        .split_once(':')
        .or_else(|| segment.split_once('='))
        .map(|(key, value)| (key.trim(), value.trim()))
        .filter(|(key, _)| !key.is_empty())
}

fn split_at_depth(value: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;

    for (index, character) in value.char_indices() {
        match character {
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            _ if character == separator && bracket_depth == 0 && brace_depth == 0 => {
                parts.push(&value[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }

    parts.push(&value[start..]);
    parts
}

fn inline_command_options(arguments: &str) -> BTreeMap<String, String> {
    arguments
        .split_whitespace()
        .filter_map(split_option)
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn damage_parts(formula: &str) -> Vec<DamagePart> {
    split_at_depth(formula, ',')
        .into_iter()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            if let Some(open) = part.rfind('[')
                && part.ends_with(']')
            {
                return DamagePart {
                    formula: part[..open].to_string(),
                    damage_type: Some(part[open + 1..part.len() - 1].to_string()),
                };
            }
            DamagePart {
                formula: part.to_string(),
                damage_type: None,
            }
        })
        .collect()
}

fn reference_display_fallback(target: &str) -> String {
    target
        .rsplit('.')
        .next()
        .unwrap_or(target)
        .replace(['-', '_'], " ")
        .trim()
        .to_string()
}

fn merge_adjacent_text(nodes: Vec<RichNode>) -> Vec<RichNode> {
    let mut merged = Vec::new();
    for node in nodes {
        match (merged.last_mut(), node) {
            (Some(RichNode::Text { text: existing }), RichNode::Text { text }) => {
                existing.push_str(&text);
            }
            (_, node) => merged.push(node),
        }
    }
    merged
}
