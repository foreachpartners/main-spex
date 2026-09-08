use std::io::IsTerminal;

#[cfg(test)]
#[path = "style_test.rs"]
mod style_test;

pub struct Style {
    enabled: bool,
}

impl Style {
    pub fn detect() -> Self {
        Self::new(std::io::stdout().is_terminal())
    }

    pub fn detect_stderr() -> Self {
        Self::new(std::io::stderr().is_terminal())
    }

    fn new(is_terminal: bool) -> Self {
        let enabled = is_terminal && std::env::var_os("NO_COLOR").is_none();
        Self { enabled }
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.enabled {
            format!("\u{1b}[{code}m{text}\u{1b}[0m")
        } else {
            text.to_string()
        }
    }

    pub fn bold(&self, text: &str) -> String {
        self.paint("1", text)
    }

    pub fn dim(&self, text: &str) -> String {
        self.paint("2", text)
    }

    pub fn red(&self, text: &str) -> String {
        self.paint("31", text)
    }

    pub fn yellow(&self, text: &str) -> String {
        self.paint("33", text)
    }

    pub fn green(&self, text: &str) -> String {
        self.paint("32", text)
    }

    pub fn cyan(&self, text: &str) -> String {
        self.paint("36", text)
    }

    pub fn hyperlink(&self, url: &str, text: &str) -> String {
        if self.enabled {
            format!("\u{1b}]8;;{url}\u{1b}\\{text}\u{1b}]8;;\u{1b}\\")
        } else {
            text.to_string()
        }
    }
}

// Emits progress to stderr while a command executes, leaving stdout for the
// command's actual output. Silent unless `--verbose` is set.
pub struct Log {
    enabled: bool,
    style: Style,
}

impl Log {
    pub fn new(verbose: bool) -> Self {
        Self {
            enabled: verbose,
            style: Style::detect_stderr(),
        }
    }

    pub fn step(&self, message: &str) {
        if self.enabled {
            eprintln!("{} {message}", self.style.dim("·"));
        }
    }
}
