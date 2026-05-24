use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    check_node(file_path, node, diagnostics);
}

fn check_node(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.resolved_tag_name == "form"
        && !node.has_submit_handler
        && has_form_controls(node)
        && !has_submit_button(node)
    {
        diagnostics.push(Diagnostic {
            code: "a11y::form_no_submit".to_string(),
            message: "Form contains inputs but has no submit button.".to_string(),
            help: Some(
                "Add a <button type=\"submit\"> or <input type=\"submit\"> so users can submit the form without relying on JavaScript."
                    .to_string(),
            ),
            file_path: file_path.to_string(),
            span: node.span,
        });
    }

    for child in &node.children {
        check_node(file_path, child, diagnostics);
    }
}

fn has_form_controls(node: &A11yNode) -> bool {
    for child in &node.children {
        if matches!(
            child.resolved_tag_name.as_str(),
            "input" | "select" | "textarea"
        ) {
            return true;
        }
        if has_form_controls(child) {
            return true;
        }
    }
    false
}

fn has_submit_button(node: &A11yNode) -> bool {
    for child in &node.children {
        if child.resolved_tag_name == "button" {
            return true;
        }
        if child.resolved_tag_name == "input"
            && child
                .input_type
                .as_deref()
                .is_some_and(|t| t == "submit" || t == "image")
        {
            return true;
        }
        if has_submit_button(child) {
            return true;
        }
    }
    false
}
