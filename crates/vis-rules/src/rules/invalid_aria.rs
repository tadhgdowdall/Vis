use vis_diagnostics::{Diagnostic, Severity};
use vis_ir::A11yNode;

use super::aria_data;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    for attr in &node.aria_attrs {
        if !aria_data::is_valid_aria_attr(attr) {
            diagnostics.push(Diagnostic {
                code: "a11y::invalid_aria".to_string(),
                message: format!("Invalid aria-* attribute: \"{attr}\"."),
                help: Some("Check the spelling of the attribute name. Valid aria-* attributes follow the WAI-ARIA specification.".to_string()),
                file_path: file_path.to_string(),
                severity: Severity::Error,
                span: node.span,
            });
        }
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}
