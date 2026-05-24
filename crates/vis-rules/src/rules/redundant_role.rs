use vis_diagnostics::{Diagnostic, Severity};
use vis_ir::A11yNode;

use super::aria_data;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if let Some(role) = &node.role {
        if role.is_empty() {
            return;
        }

        let tag = node.resolved_tag_name.as_str();

        if let Some(native) = aria_data::get_native_roles(tag)
            && native.contains(&role.as_str())
        {
            diagnostics.push(Diagnostic {
                code: "a11y::redundant_role".to_string(),
                message: format!(
                    "Redundant role=\"{role}\" on <{tag}>. This element already has that role implicitly."
                ),
                help: Some(format!(
                    "Remove role=\"{role}\". The <{tag}> element already provides this role natively."
                )),
                file_path: file_path.to_string(),
                severity: Severity::Warning,
                span: node.span,
            });
        }
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}
