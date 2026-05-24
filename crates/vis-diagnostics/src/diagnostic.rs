use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub help: Option<String>,
    pub file_path: String,
    pub span: Span,
    pub severity: Severity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
    Off,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Off => "off",
        }
    }

    pub fn is_enabled(self) -> bool {
        !matches!(self, Severity::Off)
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}
