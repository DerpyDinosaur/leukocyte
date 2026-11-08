use owo_colors::{Style, XtermColors};

use crate::cli::ui::palette::*;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Verdict {
    Block,
    Warning,
    Pass,
}

impl Verdict {
    pub fn from_score(score: u8) -> Self {
        match score {
            80..=100 => Verdict::Pass,
            50..=79 => Verdict::Warning,
            _ => Verdict::Block,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Warning => "warning",
            Verdict::Block => "blocked",
        }
    }

    pub fn xterm(self) -> u8 {
        match self {
            Verdict::Pass => GREEN,
            Verdict::Warning => YELLOW,
            Verdict::Block => RED,
        }
    }

    pub fn owo_style(self) -> Style {
        match self {
            Verdict::Pass => Style::new().color(XtermColors::from(GREEN)),
            Verdict::Warning => Style::new().color(XtermColors::from(YELLOW)),
            Verdict::Block => Style::new().color(XtermColors::from(RED)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Severity {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct FindingScoreRule {
    pub value: u8,
    pub min: u8,
    pub max: u8,
    pub weight: f32,
    /// true: a higher value means more risk (e.g. "has install script").
    ///
    /// false: a higher value means more trust (e.g. package age).
    pub inverse: bool,
}

impl Default for FindingScoreRule {
    fn default() -> Self {
        Self { value: 0, min: 0, max: 1, weight: 0.0, inverse: false }
    }
}

impl FindingScoreRule {
    pub fn script() -> Self {
        Self { weight: 0.4, inverse: true, ..Default::default() }
    }

    pub fn dependency() -> Self {
        Self { weight: 0.2, inverse: true, ..Default::default() }
    }

    pub fn age() -> Self {
        Self { max: 42, weight: 0.4, ..Default::default() }
    }
}

pub struct Finding {
    pub label: String,
    pub message: String,
    pub severity: Severity,
}

pub struct Audit {
    pub name: String,
    pub version: String,
    pub score: u8,
    pub verdict: Verdict,
    pub findings: Vec<Finding>,
}
