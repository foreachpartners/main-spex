use std::io::IsTerminal;

pub struct Style {
    enabled: bool,
}

impl Style {
    pub fn detect() -> Self {
        let enabled = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
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
}
