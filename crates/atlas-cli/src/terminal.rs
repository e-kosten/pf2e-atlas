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
