use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerminalStyle {
    enabled: bool,
}

impl TerminalStyle {
    pub(crate) fn stdout() -> Self {
        Self {
            enabled: stdout_supports_style(),
        }
    }

    #[cfg(test)]
    pub(crate) const fn enabled_for_test() -> Self {
        Self { enabled: true }
    }

    #[cfg(test)]
    pub(crate) const fn disabled() -> Self {
        Self { enabled: false }
    }

    pub(crate) fn label(self, value: &str) -> String {
        self.bold(value)
    }

    pub(crate) fn metadata(self, value: &str) -> String {
        self.dim(value)
    }

    pub(crate) fn separator(self) -> String {
        if self.enabled {
            self.dim("---")
        } else {
            "---".to_string()
        }
    }

    pub(crate) fn render_markdown(self, value: &str) -> String {
        if !self.enabled {
            return value.to_string();
        }
        value
            .lines()
            .map(|line| self.render_markdown_line(line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_markdown_line(self, line: &str) -> String {
        if line.trim() == "---" {
            return self.separator();
        }
        if let Some((marker, heading)) = markdown_heading(line) {
            let indent = &line[..line.len() - marker.len() - heading.len()];
            return format!("{indent}{}", self.bold(heading));
        }
        render_strong_spans(line, self)
    }

    fn bold(self, value: &str) -> String {
        if self.enabled {
            format!("\x1b[1m{value}\x1b[22m")
        } else {
            value.to_string()
        }
    }

    fn dim(self, value: &str) -> String {
        if self.enabled {
            format!("\x1b[2m{value}\x1b[22m")
        } else {
            value.to_string()
        }
    }
}

fn stdout_supports_style() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && std::env::var("CLICOLOR").map_or(true, |value| value != "0")
        && std::env::var("TERM").map_or(true, |value| value != "dumb")
}

fn markdown_heading(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let hashes = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &trimmed[hashes..];
    let heading = rest.strip_prefix(' ')?;
    if heading.trim().is_empty() {
        return None;
    }
    Some((&line[indent_len..indent_len + hashes + 1], heading))
}

fn render_strong_spans(line: &str, style: TerminalStyle) -> String {
    let mut output = String::new();
    let mut remaining = line;
    loop {
        let Some(start) = remaining.find("**") else {
            output.push_str(remaining);
            break;
        };
        let content_start = start + 2;
        let Some(end_relative) = remaining[content_start..].find("**") else {
            output.push_str(remaining);
            break;
        };
        let end = content_start + end_relative;
        output.push_str(&remaining[..start]);
        let content = &remaining[content_start..end];
        if content.is_empty() {
            output.push_str("****");
        } else {
            output.push_str(&style.bold(content));
        }
        remaining = &remaining[end + 2..];
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_style_keeps_markdown_syntax() {
        let style = TerminalStyle::disabled();

        assert_eq!(
            style.render_markdown("## Outcome\n\n**Success** Keep [Link](record:test:key).\n\n---"),
            "## Outcome\n\n**Success** Keep [Link](record:test:key).\n\n---"
        );
    }

    #[test]
    fn enabled_style_renders_clear_markdown_equivalents() {
        let style = TerminalStyle::enabled_for_test();

        assert_eq!(
            style.render_markdown("## Outcome\n\n**Success** Keep [Link](record:test:key).\n\n---"),
            "\u{1b}[1mOutcome\u{1b}[22m\n\n\u{1b}[1mSuccess\u{1b}[22m Keep [Link](record:test:key).\n\n\u{1b}[2m---\u{1b}[22m"
        );
    }

    #[test]
    fn unbalanced_strong_span_stays_markdown() {
        let style = TerminalStyle::enabled_for_test();

        assert_eq!(style.render_markdown("**Success text"), "**Success text");
    }
}
