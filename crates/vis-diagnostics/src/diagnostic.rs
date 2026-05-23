#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub help: Option<String>,
    pub file_path: String,
    pub span: Span,
}

#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}
