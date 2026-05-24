use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

const VALID_LANG_PATTERN: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn check(file_path: &str, node: &A11yNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.resolved_tag_name == "html" {
        match &node.lang {
            None => {
                diagnostics.push(Diagnostic {
                    code: "a11y::html_lang".to_string(),
                    message: "The <html> element is missing a lang attribute.".to_string(),
                    help: Some(
                        "Add a lang attribute to the <html> element, e.g. <html lang=\"en\">. This tells screen readers which language the page is in."
                            .to_string(),
                    ),
                    file_path: file_path.to_string(),
                    span: node.span,
                });
            }
            Some(lang) if lang.trim().is_empty() => {
                diagnostics.push(Diagnostic {
                    code: "a11y::html_lang".to_string(),
                    message: "The <html> element has an empty lang attribute.".to_string(),
                    help: Some(
                        "Set a valid language code, e.g. lang=\"en\" or lang=\"en-US\"."
                            .to_string(),
                    ),
                    file_path: file_path.to_string(),
                    span: node.span,
                });
            }
            Some(lang)
                if !lang
                    .chars()
                    .any(|c| VALID_LANG_PATTERN.contains(c)) =>
            {
                diagnostics.push(Diagnostic {
                    code: "a11y::html_lang".to_string(),
                    message: format!("The lang attribute \"{lang}\" does not look like a valid language code."),
                    help: Some(
                        "Use a valid BCP 47 language tag, e.g. \"en\", \"fr\", \"ja\", \"en-US\"."
                            .to_string(),
                    ),
                    file_path: file_path.to_string(),
                    span: node.span,
                });
            }
            _ => {}
        }
    }

    for child in &node.children {
        check(file_path, child, diagnostics);
    }
}
