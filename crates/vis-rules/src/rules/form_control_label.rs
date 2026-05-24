use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if is_labelable_control(node)
        && node
            .accessible_name
            .as_deref()
            .is_none_or(|name| name.trim().is_empty())
    {
        diagnostics.push(Diagnostic {
            code: "a11y::form_control_label".to_string(),
            message: "Form control is missing an accessible name.".to_string(),
            help: Some(
                "Prefer a native <label>. Use aria-label or aria-labelledby only when a native label is not appropriate."
                    .to_string(),
            ),
            file_path: file_path.to_string(),
            span: node.span,
        });
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}

fn is_labelable_control(node: &A11yNode) -> bool {
    match node.resolved_tag_name.as_str() {
        "select" | "textarea" => true,
        "input" => !matches!(
            node.input_type.as_deref(),
            Some("hidden" | "submit" | "reset" | "button" | "image")
        ),
        _ => false,
    }
}
