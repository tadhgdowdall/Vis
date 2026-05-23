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

impl Diagnostic {
    pub fn render(&self, source: &str) -> String {
        let position = position_for_offset(source, self.span.start);
        let line_text = source
            .lines()
            .nth(position.line.saturating_sub(1))
            .unwrap_or_default();
        let caret_width = self.span.end.saturating_sub(self.span.start).max(1);
        let caret_padding = " ".repeat(position.column.saturating_sub(1));
        let caret = "^".repeat(caret_width.min(3));

        let mut rendered = format!(
            "error[{}]\n --> {}:{}:{}\n\n{}\n\n{}\n{}",
            self.code,
            self.file_path,
            position.line,
            position.column,
            self.message,
            line_text,
            format!("{caret_padding}{caret}")
        );

        if let Some(help) = &self.help {
            rendered.push_str(&format!("\n\nhelp: {help}"));
        }

        rendered
    }
}

pub fn position_for_offset(source: &str, offset: usize) -> Position {
    let mut line = 1;
    let mut column = 1;

    for (index, ch) in source.char_indices() {
        if index >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    Position { line, column }
}
