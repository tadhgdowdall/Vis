use vis_diagnostics::{Diagnostic, Severity};
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if is_form_control(node.resolved_tag_name.as_str())
        && node.title_attr.is_some()
        && node.accessible_name.is_none()
    {
        diagnostics.push(Diagnostic {
            code: "a11y::title_label".to_string(),
            message: "The title attribute is being used as the sole label for this form control."
                .to_string(),
            help: Some(
                "The title attribute is not reliably exposed to all assistive technologies. Use <label>, aria-label, or aria-labelledby instead."
                    .to_string(),
            ),
            file_path: file_path.to_string(),
            severity: Severity::Warning,
            span: node.span,
        });
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}

fn is_form_control(tag: &str) -> bool {
    matches!(tag, "input" | "select" | "textarea")
}
