pub mod rules;

use std::collections::HashMap;

use vis_diagnostics::{Diagnostic, Severity, Span};
use vis_ir::A11yNode;

type RuleFn = fn(&str, &A11yNode, &mut Vec<Diagnostic>);

pub fn run_all(file_path: &str, nodes: &[A11yNode]) -> Vec<Diagnostic> {
    run_all_with_config(file_path, nodes, &HashMap::new())
}

pub fn run_all_with_config(
    file_path: &str,
    nodes: &[A11yNode],
    rule_config: &HashMap<String, Severity>,
) -> Vec<Diagnostic> {
    let default_severities: &[(&str, Severity)] = rules::DEFAULT_SEVERITIES;

    let effective_severities: HashMap<&str, Severity> = default_severities
        .iter()
        .map(|&(code, default)| {
            let severity = rule_config.get(code).copied().unwrap_or(default);
            (code, severity)
        })
        .collect();

    let mut diagnostics = Vec::new();

    for node in nodes {
        for (i, rule) in rules::ALL.iter().enumerate() {
            if let Some(&(code, _)) = default_severities.get(i)
                && !effective_severities[code].is_enabled()
            {
                continue;
            }
            rule(file_path, node, &mut diagnostics);
        }
    }

    for diagnostic in &mut diagnostics {
        if let Some(&severity) = effective_severities.get(diagnostic.code.as_str()) {
            diagnostic.severity = severity;
        }
    }

    diagnostics.retain(|d| {
        let suppressed = nodes
            .iter()
            .any(|node| find_node_suppression(node, d.span, d.code.as_str()));
        !suppressed
    });

    diagnostics
}

fn find_node_suppression(node: &A11yNode, span: Span, code: &str) -> bool {
    if node.span.start == span.start && node.span.end == span.end {
        return node
            .suppression_codes
            .iter()
            .any(|c| c == "all" || c == code);
    }
    node.children
        .iter()
        .any(|child| find_node_suppression(child, span, code))
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

    #[test]
    fn flags_empty_heading() {
        let parsed = parse_html("<main><h1></h1></main>").expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::empty_heading"));
    }

    #[test]
    fn allows_heading_with_text() {
        let parsed = parse_html("<main><h1>Page Title</h1></main>").expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::empty_heading"));
    }

    #[test]
    fn flags_placeholder_used_as_sole_label() {
        let parsed = parse_html(r#"<input type="text" placeholder="Enter name" />"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == "a11y::placeholder_label")
        );
    }

    #[test]
    fn allows_placeholder_with_label() {
        let parsed = parse_html(
            r#"<label for="name">Name</label><input id="name" type="text" placeholder="Enter name" />"#,
        )
        .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(
            diagnostics
                .iter()
                .all(|d| d.code != "a11y::placeholder_label")
        );
    }

    #[test]
    fn flags_title_attribute_as_sole_label() {
        let parsed =
            parse_html(r#"<input type="text" title="Search" />"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::title_label"));
    }

    #[test]
    fn allows_title_with_accessible_name() {
        let parsed = parse_html(r#"<input type="text" title="Search" aria-label="Search" />"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::title_label"));
    }

    #[test]
    fn flags_form_without_submit_button() {
        let parsed = parse_html(r#"<form><input type="text" id="name" /></form>"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::form_no_submit"));
    }

    #[test]
    fn allows_form_with_button_submit() {
        let parsed =
            parse_html(r#"<form><input type="text" /><button type="submit">Go</button></form>"#)
                .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::form_no_submit"));
    }

    #[test]
    fn allows_form_with_input_submit() {
        let parsed =
            parse_html(r#"<form><input type="text" /><input type="submit" value="Go" /></form>"#)
                .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::form_no_submit"));
    }

    #[test]
    fn allows_form_with_onsubmit_handler() {
        let parsed = parse_html(r#"<form onSubmit="handleSubmit"><input type="text" /></form>"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::form_no_submit"));
    }

    #[test]
    fn does_not_flag_empty_form() {
        let parsed = parse_html(r#"<form></form>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::form_no_submit"));
    }

    #[test]
    fn flags_invalid_aria_attribute() {
        let parsed = parse_html(r#"<div aria-labeledby="x"></div>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::invalid_aria"));
    }

    #[test]
    fn allows_valid_aria_attribute() {
        let parsed = parse_html(r#"<div aria-labelledby="x"></div>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::invalid_aria"));
    }

    #[test]
    fn flags_invalid_role() {
        let parsed = parse_html(r#"<span role="buton"></span>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::invalid_role"));
    }

    #[test]
    fn allows_valid_role() {
        let parsed = parse_html(r#"<span role="button"></span>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::invalid_role"));
    }

    #[test]
    fn flags_redundant_role_on_native_element() {
        let parsed =
            parse_html(r#"<button role="button">Save</button>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().any(|d| d.code == "a11y::redundant_role"));
    }

    #[test]
    fn allows_non_native_role_on_element() {
        let parsed =
            parse_html(r#"<button role="switch">Toggle</button>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::redundant_role"));
    }

    #[test]
    fn suppresses_diagnostic_with_data_vis_ignore() {
        let parsed =
            parse_html(r#"<div onClick="save()" data-vis-ignore="a11y::clickable_div">Save</div>"#)
                .expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::clickable_div"));
    }

    #[test]
    fn suppresses_all_diagnostics_with_data_vis_ignore_all() {
        let parsed =
            parse_html(r#"<button data-vis-ignore="all"></button>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::button_label"));
    }

    #[test]
    fn respects_aria_hidden_for_empty_heading() {
        let parsed = parse_html(r#"<h2 aria-hidden="true"></h2>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let diagnostics = run_all("example.html", &analyzed);

        assert!(diagnostics.iter().all(|d| d.code != "a11y::empty_heading"));
    }
}
