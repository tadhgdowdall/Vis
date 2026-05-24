pub mod rules;

use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

type RuleFn = fn(&str, &A11yNode, &mut Vec<Diagnostic>);

pub fn run_all(file_path: &str, nodes: &[A11yNode]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for node in nodes {
        for rule in rules::ALL {
            rule(file_path, node, &mut diagnostics);
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::run_all;
    use vis_analyzer::analyze;
    use vis_parser::parse_html;

    #[test]
    fn emits_expected_codes_for_problematic_markup() {
        let parsed = parse_html(
            r#"
            <main>
              <img src="/hero.png" />
              <div onClick="saveDraft()">Save draft</div>
              <button></button>
            </main>
            "#,
        )
        .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);
        let codes = diagnostics
            .iter()
            .map(|item| item.code.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            codes,
            vec![
                "a11y::missing_alt",
                "a11y::clickable_div",
                "a11y::button_label",
            ]
        );
    }

    #[test]
    fn does_not_emit_diagnostics_for_current_happy_path_cases() {
        let parsed = parse_html(
            r#"
            <main>
              <img src="/hero.png" alt="" />
              <button>Save draft</button>
            </main>
            "#,
        )
        .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn does_not_emit_button_diagnostic_for_aria_labelledby() {
        let parsed = parse_html(
            r#"
            <main>
              <span id="save-label">Save draft</span>
              <button aria-labelledby="save-label"></button>
            </main>
            "#,
        )
        .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "a11y::button_label")
        );
    }

    #[test]
    fn emits_semantics_diagnostic_for_clickable_span() {
        let parsed = parse_html(r#"<span onClick="openDialog()">Open dialog</span>"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "a11y::clickable_div");
    }

    #[test]
    fn emits_diagnostic_for_unlabeled_native_input() {
        let parsed = parse_html(r#"<input type="text" />"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "a11y::form_control_label");
    }

    #[test]
    fn does_not_emit_diagnostic_for_input_with_native_label() {
        let parsed = parse_html(
            r#"
            <div>
              <label for="email">Email</label>
              <input id="email" type="email" />
            </div>
            "#,
        )
        .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "a11y::form_control_label")
        );
    }

    #[test]
    fn emits_semantics_diagnostic_for_anchor_used_as_button_without_href() {
        let parsed =
            parse_html(r#"<a onClick="openDialog()">Open dialog</a>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "a11y::link_semantics");
    }

    #[test]
    fn emits_semantics_diagnostic_for_anchor_with_bogus_action_href() {
        let parsed = parse_html(r##"<a href="#" onClick="saveDraft()">Save draft</a>"##)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "a11y::link_semantics");
    }

    #[test]
    fn does_not_emit_link_semantics_diagnostic_for_real_navigation_link() {
        let parsed = parse_html(r#"<a href="/products">Products</a>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "a11y::link_semantics")
        );
    }

    #[test]
    fn emits_diagnostic_for_link_without_accessible_name() {
        let parsed = parse_html(r#"<a href="/reports"><img src="/icon.png" /></a>"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "a11y::link_label")
        );
    }

    #[test]
    fn does_not_emit_diagnostic_for_link_named_by_image_alt_text() {
        let parsed =
            parse_html(r#"<a href="/reports"><img src="/icon.png" alt="View reports" /></a>"#)
                .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "a11y::link_label")
        );
    }

    #[test]
    fn does_not_emit_button_diagnostic_for_submit_input_without_explicit_value() {
        let parsed = parse_html(r#"<input type="submit" />"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "a11y::button_label")
        );
    }

    #[test]
    fn emits_button_diagnostic_for_input_button_without_name() {
        let parsed = parse_html(r#"<input type="button" />"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "a11y::button_label")
        );
    }
}
