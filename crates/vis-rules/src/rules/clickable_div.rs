use vis_diagnostics::{Diagnostic, Severity};
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if matches!(node.resolved_tag_name.as_str(), "div" | "span")
        && node.has_click_handler
        && !node.focusable
    {
        diagnostics.push(Diagnostic {
            code: "a11y::clickable_div".to_string(),
            message: "Non-native interactive elements should use native HTML semantics.".to_string(),
            help: Some(
                "Use a native <button> for actions. If you keep a custom element, add keyboard and focus support."
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
