use vis_diagnostics::{Diagnostic, Severity};
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.resolved_tag_name == "title"
        && node
            .accessible_name
            .as_deref()
            .is_none_or(|name| name.trim().is_empty())
    {
        diagnostics.push(Diagnostic {
            code: "a11y::page_title".to_string(),
            message: "The <title> element is missing or empty.".to_string(),
            help: Some(
                "Every page should have a descriptive <title> that identifies its purpose. This appears in browser tabs and search results."
                    .to_string(),
            ),
            file_path: file_path.to_string(),
            severity: Severity::Error,
            span: node.span,
        });
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}
