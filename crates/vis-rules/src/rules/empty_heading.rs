use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.heading_level.is_some() && node.accessible_name.as_deref().is_none_or(str::is_empty) {
        diagnostics.push(Diagnostic {
            code: "a11y::empty_heading".to_string(),
            message: "Heading element has no content.".to_string(),
            help: Some(
                "Add text content to the heading so screen reader users can navigate the page outline."
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
