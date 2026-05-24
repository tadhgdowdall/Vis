use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if is_form_control(node.resolved_tag_name.as_str())
        && node.placeholder.is_some()
        && node.accessible_name.is_none()
    {
        diagnostics.push(Diagnostic {
            code: "a11y::placeholder_label".to_string(),
            message: "Placeholder text is being used as the sole label for this form control."
                .to_string(),
            help: Some(
                "Placeholders are not a substitute for labels. Add a visible <label> element associated with this control."
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

fn is_form_control(tag: &str) -> bool {
    matches!(tag, "input" | "select" | "textarea")
}
