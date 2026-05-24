use vis_diagnostics::{Diagnostic, Severity};
use vis_ir::A11yNode;

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.resolved_tag_name == "input"
        && node.autocomplete.is_none()
        && let Some(input_type) = node.input_type.as_deref()
        && needs_autocomplete(input_type)
    {
        diagnostics.push(Diagnostic {
            code: "a11y::autocomplete".to_string(),
            message: format!(
                "Input type=\"{input_type}\" is missing an autocomplete attribute."
            ),
            help: Some(
                "Add an autocomplete attribute to help browsers fill in the user's information, e.g. autocomplete=\"email\" or autocomplete=\"name\"."
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

fn needs_autocomplete(input_type: &str) -> bool {
    matches!(
        input_type,
        "email"
            | "tel"
            | "url"
            | "name"
            | "honorific-prefix"
            | "given-name"
            | "additional-name"
            | "family-name"
            | "honorific-suffix"
            | "nickname"
            | "username"
            | "new-password"
            | "current-password"
            | "organization"
            | "street-address"
            | "address-line1"
            | "address-line2"
            | "address-line3"
            | "address-level1"
            | "address-level2"
            | "address-level3"
            | "address-level4"
            | "country"
            | "country-name"
            | "postal-code"
            | "cc-name"
            | "cc-number"
            | "cc-exp"
            | "cc-exp-month"
            | "cc-exp-year"
            | "cc-csc"
            | "cc-type"
            | "transaction-currency"
            | "transaction-amount"
            | "bday"
            | "bday-day"
            | "bday-month"
            | "bday-year"
            | "sex"
            | "impp"
            | "language"
            | "photo"
    )
}
