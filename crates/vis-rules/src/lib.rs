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

    #[test]
    fn flags_positive_tabindex() {
        let parsed = parse_html(r#"<div tabindex="3">Focusable</div>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == "a11y::tabindex_misuse")
        );
    }

    #[test]
    fn allows_tabindex_zero_and_negative() {
        let parsed = parse_html(r#"<div tabindex="0">A</div><div tabindex="-1">B</div>"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .all(|d| d.code != "a11y::tabindex_misuse")
        );
    }

    #[test]
    fn flags_heading_skip_from_h1_to_h3() {
        let parsed = parse_html("<main><h1>A</h1><h3>B</h3></main>").expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == "a11y::heading_hierarchy")
        );
    }

    #[test]
    fn allows_consecutive_headings() {
        let parsed =
            parse_html("<main><h1>A</h1><h2>B</h2><h3>C</h3></main>").expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .all(|d| d.code != "a11y::heading_hierarchy")
        );
    }

    #[test]
    fn flags_first_heading_not_h1() {
        let parsed = parse_html("<main><h2>Title</h2></main>").expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == "a11y::heading_hierarchy")
        );
    }

    #[test]
    fn flags_html_missing_lang() {
        let parsed =
            parse_html("<html><head></head><body></body></html>").expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::html_lang"));
    }

    #[test]
    fn allows_html_with_valid_lang() {
        let parsed = parse_html(r#"<html lang="en"><head></head><body></body></html>"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::html_lang"));
    }

    #[test]
    fn flags_empty_lang() {
        let parsed = parse_html(r#"<html lang=""><head></head><body></body></html>"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::html_lang"));
    }

    #[test]
    fn flags_empty_title() {
        let parsed = parse_html("<html><head><title></title></head><body></body></html>")
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::page_title"));
    }

    #[test]
    fn allows_nonempty_title() {
        let parsed = parse_html("<html><head><title>My Page</title></head><body></body></html>")
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::page_title"));
    }

    #[test]
    fn flags_email_input_without_autocomplete() {
        let parsed = parse_html(r#"<input type="email" />"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::autocomplete"));
    }

    #[test]
    fn allows_email_input_with_autocomplete() {
        let parsed = parse_html(r#"<input type="email" autocomplete="email" />"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::autocomplete"));
    }

    #[test]
    fn allows_text_input_without_autocomplete() {
        let parsed = parse_html(r#"<input type="text" />"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::autocomplete"));
    }
}
