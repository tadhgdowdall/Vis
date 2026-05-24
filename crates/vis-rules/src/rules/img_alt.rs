use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.resolved_tag_name == "img" && node.alt_text.is_none() {
        diagnostics.push(Diagnostic {
            code: "a11y::missing_alt".to_string(),
            message: "Image missing alt text.".to_string(),
            help: Some("Add alt text, or alt=\"\" if the image is decorative.".to_string()),
            file_path: file_path.to_string(),
            span: node.span,
        });
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}
