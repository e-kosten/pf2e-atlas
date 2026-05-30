#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum HtmlNode {
    Text(String),
    Element {
        name: String,
        children: Vec<HtmlNode>,
    },
}

pub(super) fn parse_html_fragment(value: &str) -> Vec<HtmlNode> {
    let mut root = Vec::new();
    let mut stack = Vec::new();
    let mut offset = 0;

    while offset < value.len() {
        let rest = &value[offset..];
        if rest.starts_with('<')
            && let Some(close_relative) = rest.find('>')
        {
            let raw_tag = &rest[1..close_relative];
            offset += close_relative + 1;
            let tag = parse_tag(raw_tag);
            match tag {
                ParsedTag::Start { name, self_closing } => {
                    if is_void_tag(&name) || self_closing {
                        push_child(
                            &mut root,
                            &mut stack,
                            HtmlNode::Element {
                                name,
                                children: Vec::new(),
                            },
                        );
                    } else {
                        stack.push(OpenElement {
                            name,
                            children: Vec::new(),
                        });
                    }
                }
                ParsedTag::End { name } => close_element(&mut root, &mut stack, &name),
                ParsedTag::CommentOrDirective => {}
            }
            continue;
        }

        let next_tag = rest.find('<').unwrap_or(rest.len());
        if next_tag > 0 {
            push_child(
                &mut root,
                &mut stack,
                HtmlNode::Text(decode_entities(&rest[..next_tag])),
            );
            offset += next_tag;
        } else {
            let Some(character) = rest.chars().next() else {
                break;
            };
            push_child(&mut root, &mut stack, HtmlNode::Text(character.to_string()));
            offset += character.len_utf8();
        }
    }

    while let Some(element) = stack.pop() {
        push_child(
            &mut root,
            &mut stack,
            HtmlNode::Element {
                name: element.name,
                children: element.children,
            },
        );
    }

    root.into_iter()
        .filter(|node| !node_is_empty_text(node))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpenElement {
    name: String,
    children: Vec<HtmlNode>,
}

fn push_child(root: &mut Vec<HtmlNode>, stack: &mut [OpenElement], child: HtmlNode) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(child);
    } else {
        root.push(child);
    }
}

fn close_element(root: &mut Vec<HtmlNode>, stack: &mut Vec<OpenElement>, name: &str) {
    let Some(position) = stack.iter().rposition(|element| element.name == name) else {
        return;
    };

    while stack.len() > position {
        let Some(element) = stack.pop() else {
            break;
        };
        push_child(
            root,
            stack,
            HtmlNode::Element {
                name: element.name,
                children: element.children,
            },
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedTag {
    Start { name: String, self_closing: bool },
    End { name: String },
    CommentOrDirective,
}

fn parse_tag(raw_tag: &str) -> ParsedTag {
    let tag = raw_tag.trim();
    if tag.starts_with('!') || tag.starts_with('?') {
        return ParsedTag::CommentOrDirective;
    }
    if let Some(rest) = tag.strip_prefix('/') {
        return ParsedTag::End {
            name: tag_name(rest).to_string(),
        };
    }

    ParsedTag::Start {
        name: tag_name(tag).to_string(),
        self_closing: tag.ends_with('/'),
    }
}

fn tag_name(tag: &str) -> &str {
    tag.split(|character: char| character.is_whitespace() || character == '/')
        .next()
        .unwrap_or_default()
        .trim()
}

fn is_void_tag(name: &str) -> bool {
    matches!(name, "br" | "hr" | "img" | "input" | "meta" | "link")
}

fn node_is_empty_text(node: &HtmlNode) -> bool {
    matches!(node, HtmlNode::Text(text) if text.trim().is_empty())
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
