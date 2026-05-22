pub struct Diagnostic {
    pub code: String,
}

pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub struct Position {
    pub line: usize,
    pub column: usize,
}
