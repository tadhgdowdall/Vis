use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.tag_name == "a"
        && node.href.is_some()
        && node
            .accessible_name
            .as_deref()
            .is_none_or(|name| name.trim().is_empty())
    {
        diagnostics.push(Diagnostic {
            code: "a11y::link_label".to_string(),
            message: "Link is missing an accessible name.".to_string(),
            help: Some(
                "Use descriptive link text. For image-only links, provide meaningful alt text on the image."
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
