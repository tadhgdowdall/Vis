use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.tag_name == "a"
        && node.has_click_handler
        && looks_like_action_anchor(node.href.as_deref())
    {
        diagnostics.push(Diagnostic {
            code: "a11y::link_semantics".to_string(),
            message: "Anchors should be used for navigation, not button-like actions.".to_string(),
            help: Some(
                "Use <button> for actions. Use <a href=\"...\"> only when the element navigates to a real destination."
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

fn looks_like_action_anchor(href: Option<&str>) -> bool {
    match href.map(str::trim) {
        None => true,
        Some("#") => true,
        Some(value) if value.eq_ignore_ascii_case("javascript:void(0)") => true,
        Some(value) if value.eq_ignore_ascii_case("javascript:void(0);") => true,
        Some(value) if value.to_ascii_lowercase().starts_with("javascript:") => true,
        _ => false,
    }
}
