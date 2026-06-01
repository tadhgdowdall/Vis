use vis_diagnostics::{Diagnostic, Severity};
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if matches!(node.resolved_tag_name.as_str(), "div" | "span")
        && node.has_click_handler
        && !node.has_keyboard_handler
        && has_custom_interactive_semantics(node)
    {
        diagnostics.push(Diagnostic {
            code: "a11y::keyboard_handler".to_string(),
            message: "Interactive element has a click handler but no keyboard handler.".to_string(),
            help: Some(
                "Add onKeyDown or onKeyUp to handle keyboard interaction, or use a native <button> which is keyboard accessible by default."
                    .to_string(),
            ),
            file_path: file_path.to_string(),
            severity: Severity::Error,
            span: node.span,
        });
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}

fn has_custom_interactive_semantics(node: &A11yNode) -> bool {
    node.focusable
        || node.role.as_deref().is_some_and(|role| {
            matches!(
                role,
                "button"
                    | "checkbox"
                    | "link"
                    | "menuitem"
                    | "menuitemcheckbox"
                    | "menuitemradio"
                    | "option"
                    | "radio"
                    | "switch"
                    | "tab"
                    | "treeitem"
            )
        })
}
