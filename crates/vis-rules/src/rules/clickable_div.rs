use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.tag_name == "div" && node.has_click_handler && !node.focusable {
        diagnostics.push(Diagnostic {
            code: "a11y::clickable_div".to_string(),
            message: "Interactive divs must be keyboard accessible.".to_string(),
            help: Some("Use a <button> or add keyboard/focus support.".to_string()),
            file_path: file_path.to_string(),
            span: node.span,
        });
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}
