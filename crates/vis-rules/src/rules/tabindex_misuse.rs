use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if let Some(tab_index) = &node.tab_index
        && let Ok(value) = tab_index.trim().parse::<i32>()
        && value > 0
    {
        diagnostics.push(Diagnostic {
            code: "a11y::tabindex_misuse".to_string(),
            message: format!("tabindex=\"{value}\" breaks the natural focus order."),
            help: Some(
                "Prefer tabindex=\"0\" to include an element in the natural tab order, or tabindex=\"-1\" to make it programmatically focusable without adding it to the order."
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
