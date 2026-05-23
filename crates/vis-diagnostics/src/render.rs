use crate::{Diagnostic, Position};

pub fn render_diagnostic(diagnostic: &Diagnostic, source: &str) -> String {
    let position = position_for_offset(source, diagnostic.span.start);
    let line_text = source
        .lines()
        .nth(position.line.saturating_sub(1))
        .unwrap_or_default();
    let caret_width = diagnostic
        .span
        .end
        .saturating_sub(diagnostic.span.start)
        .max(1);
    let caret_padding = " ".repeat(position.column.saturating_sub(1));
    let caret = "^".repeat(caret_width.min(3));
    let marker = format!("{caret_padding}{caret}");

    let mut rendered = format!(
        "error[{}]\n --> {}:{}:{}\n\n{}\n\n{}\n{}",
        diagnostic.code,
        diagnostic.file_path,
        position.line,
        position.column,
        diagnostic.message,
        line_text,
        marker
    );

    if let Some(help) = &diagnostic.help {
        rendered.push_str(&format!("\n\nhelp: {help}"));
    }

    rendered
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
