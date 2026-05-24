use serde::Serialize;

use crate::{Diagnostic, position_for_offset};

#[derive(Serialize)]
struct DiagnosticOutput {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<String>,
    file: String,
    span: crate::Span,
    line: usize,
    column: usize,
}

pub fn render_diagnostics_json<'a>(
    reports: impl IntoIterator<Item = &'a (&'a str, &'a [Diagnostic])>,
) -> String {
    let outputs: Vec<DiagnosticOutput> = reports
        .into_iter()
        .flat_map(|(source, diagnostics)| {
            diagnostics.iter().map(|d| {
                let position = position_for_offset(source, d.span.start);
                DiagnosticOutput {
                    code: d.code.clone(),
                    message: d.message.clone(),
                    help: d.help.clone(),
                    file: d.file_path.clone(),
                    span: d.span,
                    line: position.line,
                    column: position.column,
                }
            })
        })
        .collect();

    serde_json::to_string_pretty(&outputs).unwrap_or_else(|_| "[]".to_string())
}
