use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if is_button_like(node)
        && node.interactive
        && node
            .accessible_name
            .as_deref()
            .is_none_or(|name| name.trim().is_empty())
    {
        diagnostics.push(Diagnostic {
            code: "a11y::button_label".to_string(),
            message: "Button is missing an accessible name.".to_string(),
            help: Some(
                "Prefer visible button text. If the button has no visible text, use aria-label or aria-labelledby."
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

fn is_button_like(node: &A11yNode) -> bool {
    match node.tag_name.as_str() {
        "button" => true,
        "input" => matches!(
            node.input_type.as_deref(),
            Some("button" | "image" | "submit" | "reset")
        ),
        _ => false,
    }
}
