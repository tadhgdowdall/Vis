use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    let mut last_heading: Option<u8> = None;
    check_node(node, &mut last_heading, file_path, diagnostics);
}

fn check_node(
    node: &A11yNode,
    last_heading: &mut Option<u8>,
    file_path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(level) = node.heading_level {
        if let Some(current) = *last_heading {
            if level > current + 1 {
                diagnostics.push(Diagnostic {
                    code: "a11y::heading_hierarchy".to_string(),
                    message: format!(
                        "Heading level skipped from h{current} to h{level}."
                    ),
                    help: Some(
                        "Heading levels should not be skipped. Add an intermediate heading or adjust the levels to form a logical outline."
                            .to_string(),
                    ),
                    file_path: file_path.to_string(),
                    span: node.span,
                });
            }
        } else if level > 1 {
            diagnostics.push(Diagnostic {
                code: "a11y::heading_hierarchy".to_string(),
                message: format!("Page starts with h{level} instead of h1."),
                help: Some(
                    "The first heading on a page should be an h1. Use h1 for the primary page title."
                        .to_string(),
                ),
                file_path: file_path.to_string(),
                span: node.span,
            });
        }
        *last_heading = Some(level);
    }

    for child in &node.children {
        check_node(child, last_heading, file_path, diagnostics);
    }
}
