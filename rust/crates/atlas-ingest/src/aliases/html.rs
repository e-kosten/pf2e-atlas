pub(super) fn html_cells(row_html: &str) -> Vec<String> {
    let mut cells = html_elements(row_html, "td");
    cells.extend(html_elements(row_html, "th"));
    cells
}

pub(super) fn html_elements(html: &str, tag_name: &str) -> Vec<String> {
    let lower = html.to_lowercase();
    let open_prefix = format!("<{tag_name}");
    let close_tag = format!("</{tag_name}>");
    let mut elements = Vec::new();
    let mut offset = 0;
    while let Some(open_relative) = lower[offset..].find(&open_prefix) {
        let open = offset + open_relative;
        let Some(open_end_relative) = lower[open..].find('>') else {
            break;
        };
        let content_start = open + open_end_relative + 1;
        let Some(close_relative) = lower[content_start..].find(&close_tag) else {
            break;
        };
        let close = content_start + close_relative;
        elements.push(html[content_start..close].to_string());
        offset = close + close_tag.len();
    }
    elements
}

pub(super) fn html_text(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            '@' if !in_tag && chars.peek().is_some_and(|next| *next == 'U') => {
                for next in chars.by_ref() {
                    if next == ']' {
                        break;
                    }
                }
                if chars.peek().is_some_and(|next| *next == '{') {
                    let _ = chars.next();
                    for display in chars.by_ref() {
                        if display == '}' {
                            break;
                        }
                        output.push(display);
                    }
                }
                output.push(' ');
            }
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}
