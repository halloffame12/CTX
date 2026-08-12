//! Terminal rendering helpers + JSON-mode output.

use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutMode {
    Human,
    Json,
}

#[derive(Debug, Clone)]
pub struct Term {
    pub mode: OutMode,
    pub color: bool,
    pub quiet: bool,
}

impl Term {
    pub fn new(json: bool, no_color: bool, quiet: bool) -> Term {
        let ci = std::env::var("CI")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let tty = std::io::stdout().is_terminal();
        let color = !no_color && !ci && tty;
        Term {
            mode: if json { OutMode::Json } else { OutMode::Human },
            color,
            quiet,
        }
    }

    pub fn is_json(&self) -> bool {
        self.mode == OutMode::Json
    }

    pub fn p(&self, text: &str) {
        if !self.quiet {
            println!("{text}");
        }
    }

    pub fn e(&self, text: &str) {
        eprintln!("{text}");
    }

    pub fn section(&self, title: &str) {
        if !self.quiet && !self.is_json() {
            let line = "─".repeat(title.len().max(8));
            let line = self.style(Default::BOLD, &line);
            let title = self.style(Default::BOLD, title);
            println!();
            println!("{line}\n{title}\n{line}");
        }
    }

    pub fn bullet(&self, indent: usize, text: &str) {
        if !self.quiet && !self.is_json() {
            println!("{}• {}", "  ".repeat(indent), text);
        }
    }

    pub fn ok(&self, text: &str) -> String {
        format!("{} {text}", self.style(Default::GREEN, "✓"))
    }

    pub fn style(&self, code: u8, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

#[allow(non_snake_case)]
pub mod Default {
    pub const BOLD: u8 = 1;
    pub const DIM: u8 = 2;
    pub const UNDERLINE: u8 = 4;
    pub const RED: u8 = 31;
    pub const GREEN: u8 = 32;
    pub const YELLOW: u8 = 33;
    pub const BLUE: u8 = 34;
    pub const MAGENTA: u8 = 35;
    pub const CYAN: u8 = 36;
}

pub fn emit_json(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_default()
    );
}
