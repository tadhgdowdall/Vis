use std::collections::HashMap;

use vis_ir::A11yNode;
use vis_parser::ParsedNode;

pub fn analyze(nodes: &[ParsedNode]) -> Vec<A11yNode> {
    let context = AnalysisContext::new(nodes);

    nodes
        .iter()
        .map(|node| analyze_node(node, &context))
        .collect()
}

struct AnalysisContext {
    labels_by_id: HashMap<String, String>,
    labels_by_control_id: HashMap<String, String>,
    labels_by_control_span: HashMap<usize, String>,
}

impl AnalysisContext {
    fn new(nodes: &[ParsedNode]) -> Self {
        let mut labels_by_id = HashMap::new();
        let mut labels_by_control_id = HashMap::new();
        let mut labels_by_control_span = HashMap::new();

        for node in nodes {
            collect_labels_by_id(node, &mut labels_by_id);
        }

        for node in nodes {
            collect_control_labels(node, &mut labels_by_control_id, &mut labels_by_control_span);
        }

        Self {
            labels_by_id,
            labels_by_control_id,
            labels_by_control_span,
        }
    }
}

fn collect_labels_by_id(node: &ParsedNode, labels_by_id: &mut HashMap<String, String>) {
    if let Some(id) = node.attribute_value("id") {
        let text = normalize_text(&node.text());
        if !text.is_empty() {
            labels_by_id.insert(id.to_string(), text);
        }
    }

    for child in &node.children {
        collect_labels_by_id(child, labels_by_id);
    }
}

fn collect_control_labels(
    node: &ParsedNode,
    labels_by_control_id: &mut HashMap<String, String>,
    labels_by_control_span: &mut HashMap<usize, String>,
) {
    if node.tag_name.eq_ignore_ascii_case("label") {
        let label_text = normalize_text(&node.text());

        if !label_text.is_empty() {
            if let Some(target_id) = node
                .attribute_value("for")
                .or_else(|| node.attribute_value("htmlFor"))
            {
                labels_by_control_id.insert(target_id.to_string(), label_text.clone());
            }

            let labeled_controls = collect_descendant_controls(node);
            if labeled_controls.len() == 1 {
                labels_by_control_span.insert(labeled_controls[0].span.start, label_text);
            }
        }
    }

    for child in &node.children {
        collect_control_labels(child, labels_by_control_id, labels_by_control_span);
    }
}

fn collect_descendant_controls(node: &ParsedNode) -> Vec<&ParsedNode> {
    let mut controls = Vec::new();

    for child in &node.children {
        if is_labelable_control(&child.tag_name) {
            controls.push(child);
        }

        controls.extend(collect_descendant_controls(child));
    }

    controls
}

fn analyze_node(node: &ParsedNode, context: &AnalysisContext) -> A11yNode {
    let tag_name = node.tag_name.to_ascii_lowercase();
    let input_type = node
        .attribute_value("type")
        .map(|value| value.to_ascii_lowercase());
    let has_click_handler = node
        .attributes
        .iter()
        .any(|attribute| attribute.name.eq_ignore_ascii_case("onclick"));
    let href = node.attribute_value("href");
    let tab_index = node.attribute_value("tabindex");
    let aria_labelledby = node.attribute_value("aria-labelledby");
    let aria_label = node.attribute_value("aria-label").map(normalize_text);
    let text_content = normalize_text(&node.text());
    let alt_text = node.attribute_value("alt").map(normalize_text);

    let interactive = matches!(
        tag_name.as_str(),
        "button" | "input" | "select" | "textarea"
    ) || (tag_name == "a" && href.is_some())
        || has_click_handler;

    let focusable = matches!(
        tag_name.as_str(),
        "button" | "input" | "select" | "textarea"
    ) || (tag_name == "a" && href.is_some())
        || tab_index.is_some_and(|value| value.trim() != "-1");

    let accessible_name = match tag_name.as_str() {
        "button" => (!text_content.is_empty())
            .then_some(text_content.clone())
            .or_else(|| accessible_name_from_references(aria_labelledby, context))
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
        "input" | "select" | "textarea" => native_label_for_control(node, context)
            .or_else(|| accessible_name_from_references(aria_labelledby, context))
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
        "img" => alt_text.clone(),
        _ => accessible_name_from_references(aria_labelledby, context)
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
    };

    A11yNode {
        tag_name,
        href: href.map(str::to_string),
        input_type,
        interactive,
        focusable,
        accessible_name,
        has_click_handler,
        alt_text,
        span: node.span,
        children: node
            .children
            .iter()
            .map(|child| analyze_node(child, context))
            .collect(),
    }
}

fn accessible_name_from_references(
    aria_labelledby: Option<&str>,
    context: &AnalysisContext,
) -> Option<String> {
    let ids = aria_labelledby?;
    let label = ids
        .split_whitespace()
        .filter_map(|id| context.labels_by_id.get(id))
        .map(|text| text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let label = normalize_text(&label);

    (!label.is_empty()).then_some(label)
}

fn native_label_for_control(node: &ParsedNode, context: &AnalysisContext) -> Option<String> {
    if let Some(id) = node.attribute_value("id")
        && let Some(label) = context.labels_by_control_id.get(id)
    {
        return Some(label.clone());
    }

    context
        .labels_by_control_span
        .get(&node.span.start)
        .cloned()
}

fn is_labelable_control(tag_name: &str) -> bool {
    matches!(tag_name, "input" | "select" | "textarea")
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::analyze;
    use vis_parser::parse_html;

    #[test]
    fn derives_button_name_from_visible_text() {
        let parsed = parse_html("<button>Save draft</button>").expect("html should parse");

        let analyzed = analyze(&parsed);
        let button = &analyzed[0];

        assert_eq!(button.tag_name, "button");
        assert!(button.interactive);
        assert!(button.focusable);
        assert_eq!(button.accessible_name.as_deref(), Some("Save draft"));
    }

    #[test]
    fn uses_aria_label_when_present() {
        let parsed =
            parse_html(r#"<button aria-label="Close"></button>"#).expect("html should parse");

        let analyzed = analyze(&parsed);

        assert_eq!(analyzed[0].accessible_name.as_deref(), Some("Close"));
    }

    #[test]
    fn uses_aria_labelledby_when_present() {
        let parsed = parse_html(
            r#"
            <div>
              <span id="close-label">Close dialog</span>
              <button aria-labelledby="close-label"></button>
            </div>
            "#,
        )
        .expect("html should parse");

        let analyzed = analyze(&parsed);
        let button = &analyzed[0].children[1];

        assert_eq!(button.accessible_name.as_deref(), Some("Close dialog"));
    }

    #[test]
    fn uses_native_label_for_named_input() {
        let parsed = parse_html(
            r#"
            <div>
              <label for="email">Email address</label>
              <input id="email" type="email" />
            </div>
            "#,
        )
        .expect("html should parse");

        let analyzed = analyze(&parsed);
        let input = &analyzed[0].children[1];

        assert_eq!(input.accessible_name.as_deref(), Some("Email address"));
    }

    #[test]
    fn uses_wrapped_label_for_named_input() {
        let parsed =
            parse_html(r#"<label>Name <input type="text" /></label>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let input = &analyzed[0].children[0];

        assert_eq!(input.accessible_name.as_deref(), Some("Name"));
    }

    #[test]
    fn preserves_href_for_anchor_nodes() {
        let parsed = parse_html(r#"<a href="/account">Account</a>"#).expect("html should parse");

        let analyzed = analyze(&parsed);
        let anchor = &analyzed[0];

        assert_eq!(anchor.href.as_deref(), Some("/account"));
        assert!(anchor.focusable);
    }

    #[test]
    fn marks_clickable_div_as_interactive_but_not_focusable() {
        let parsed = parse_html(r#"<div onClick="saveDraft()">Save draft</div>"#)
            .expect("html should parse");

        let analyzed = analyze(&parsed);
        let div = &analyzed[0];

        assert_eq!(div.tag_name, "div");
        assert!(div.interactive);
        assert!(!div.focusable);
        assert!(div.has_click_handler);
    }
}
