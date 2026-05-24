use vis_diagnostics::{Diagnostic, Severity};
use vis_ir::A11yNode;

use super::aria_data;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if let Some(role) = &node.role
        && !role.is_empty()
        && !aria_data::is_valid_role(role)
    {
        diagnostics.push(Diagnostic {
            code: "a11y::invalid_role".to_string(),
            message: format!("Invalid ARIA role: \"{role}\"."),
            help: Some("Use a valid ARIA role from the WAI-ARIA specification, e.g. \"button\", \"navigation\", \"alert\".".to_string()),
            file_path: file_path.to_string(),
            severity: Severity::Error,
            span: node.span,
        });
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}
