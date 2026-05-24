use std::collections::HashMap;

use vis_ir::A11yNode;
use vis_parser::ParsedNode;

pub fn analyze(nodes: &[ParsedNode]) -> Vec<A11yNode> {
    analyze_with_map(nodes, None)
}

pub fn analyze_with_map(
    nodes: &[ParsedNode],
    component_map: Option<HashMap<String, String>>,
) -> Vec<A11yNode> {
    let context = AnalysisContext::new(nodes, component_map);

    nodes
        .iter()
        .map(|node| analyze_node(node, None, &context))
        .collect()
}

struct AnalysisContext {
    labels_by_id: HashMap<String, String>,
    labels_by_control_id: HashMap<String, String>,
    component_map: HashMap<String, String>,
}

impl AnalysisContext {
    fn new(nodes: &[ParsedNode], component_map: Option<HashMap<String, String>>) -> Self {
        let mut labels_by_id = HashMap::new();
        let mut labels_by_control_id = HashMap::new();

        for node in nodes {
            collect_labels_by_id(node, &mut labels_by_id);
        }

        for node in nodes {
            collect_control_labels(
                node,
                &mut labels_by_control_id,
                component_map.as_ref().unwrap_or(&HashMap::new()),
            );
        }

        Self {
            labels_by_id,
            labels_by_control_id,
            component_map: component_map.unwrap_or_default(),
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
    component_map: &HashMap<String, String>,
) {
    let resolved = resolve_element(&node.tag_name.to_ascii_lowercase(), component_map);
    if resolved == "label" {
        let label_text = normalize_text(&node.text());

        if !label_text.is_empty()
            && let Some(target_id) = node
                .attribute_value("for")
                .or_else(|| node.attribute_value("htmlFor"))
        {
            labels_by_control_id.insert(target_id.to_string(), label_text);
        }
    }

    for child in &node.children {
        collect_control_labels(child, labels_by_control_id, component_map);
    }
}

fn analyze_node(
    node: &ParsedNode,
    wrapping_label: Option<&str>,
    context: &AnalysisContext,
) -> A11yNode {
    let tag_name = node.tag_name.to_ascii_lowercase();
    let resolved_tag = resolve_element(&tag_name, &context.component_map);
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
    let input_value = node.attribute_value("value").map(normalize_text);
    let text_content = normalize_text(&node.text());
    let alt_text = node.attribute_value("alt").map(normalize_text);
    let lang = node.attribute_value("lang");
    let autocomplete = node.attribute_value("autocomplete");
    let placeholder = node.attribute_value("placeholder").map(normalize_text);
    let title_attr = node.attribute_value("title").map(normalize_text);
    let has_submit_handler = node
        .attributes
        .iter()
        .any(|attribute| attribute.name.eq_ignore_ascii_case("onsubmit"));
    let heading_level = heading_level_from_tag(&resolved_tag);

    let child_wrapping_label: Option<String> = if resolved_tag == "label" {
        let label_text = normalize_text(&node.text());
        if !label_text.is_empty() {
            Some(label_text)
        } else {
            wrapping_label.map(|s| s.to_string())
        }
    } else {
        wrapping_label.map(|s| s.to_string())
    };
    let child_wrapping_label: Option<&str> = child_wrapping_label.as_deref();

    let interactive = matches!(
        resolved_tag.as_str(),
        "button" | "input" | "select" | "textarea"
    ) || (resolved_tag == "a" && href.is_some())
        || has_click_handler;

    let focusable = matches!(
        resolved_tag.as_str(),
        "button" | "input" | "select" | "textarea"
    ) || (resolved_tag == "a" && href.is_some())
        || tab_index.is_some_and(|value| value.trim() != "-1");

    let accessible_name = match resolved_tag.as_str() {
        "button" => (!text_content.is_empty())
            .then_some(text_content.clone())
            .or_else(|| accessible_name_from_references(aria_labelledby, context))
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
        "input" => accessible_name_for_input(
            node,
            wrapping_label,
            input_type.as_deref(),
            input_value,
            alt_text.clone(),
            aria_labelledby,
            aria_label,
            context,
        ),
        "select" | "textarea" => native_label_for_control(node, wrapping_label, context)
            .or_else(|| accessible_name_from_references(aria_labelledby, context))
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
        "img" => alt_text.clone(),
        "a" => (!text_content.is_empty())
            .then_some(text_content.clone())
            .or_else(|| accessible_name_from_descendant_images(node, &context.component_map))
            .or_else(|| accessible_name_from_references(aria_labelledby, context))
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
        _ => (!text_content.is_empty())
            .then_some(text_content.clone())
            .or_else(|| accessible_name_from_references(aria_labelledby, context))
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
    };

    A11yNode {
        tag_name,
        resolved_tag_name: resolved_tag,
        href: href.map(str::to_string),
        input_type,
        interactive,
        focusable,
        accessible_name,
        has_click_handler,
        alt_text,
        tab_index: tab_index.map(str::to_string),
        heading_level,
        lang: lang.map(str::to_string),
        autocomplete: autocomplete.map(str::to_string),
        placeholder,
        title_attr,
        has_submit_handler,
        span: node.span,
        children: node
            .children
            .iter()
            .map(|child| analyze_node(child, child_wrapping_label, context))
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

#[allow(clippy::too_many_arguments)]
fn accessible_name_for_input(
    node: &ParsedNode,
    wrapping_label: Option<&str>,
    input_type: Option<&str>,
    input_value: Option<String>,
    alt_text: Option<String>,
    aria_labelledby: Option<&str>,
    aria_label: Option<String>,
    context: &AnalysisContext,
) -> Option<String> {
    match input_type {
        Some("submit") => input_value
            .filter(|value| !value.is_empty())
            .or_else(|| Some("Submit".to_string())),
        Some("reset") => input_value
            .filter(|value| !value.is_empty())
            .or_else(|| Some("Reset".to_string())),
        Some("button") => input_value
            .filter(|value| !value.is_empty())
            .or_else(|| accessible_name_from_references(aria_labelledby, context))
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
        Some("image") => alt_text
            .filter(|text| !text.is_empty())
            .or_else(|| accessible_name_from_references(aria_labelledby, context))
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
        _ => native_label_for_control(node, wrapping_label, context)
            .or_else(|| accessible_name_from_references(aria_labelledby, context))
            .or_else(|| aria_label.filter(|label| !label.is_empty())),
    }
}

fn accessible_name_from_descendant_images(
    node: &ParsedNode,
    component_map: &HashMap<String, String>,
) -> Option<String> {
    let label = collect_descendant_image_alt_text(node, component_map).join(" ");
    let label = normalize_text(&label);

    (!label.is_empty()).then_some(label)
}

fn collect_descendant_image_alt_text(
    node: &ParsedNode,
    component_map: &HashMap<String, String>,
) -> Vec<String> {
    let mut labels = Vec::new();

    for child in &node.children {
        let resolved = resolve_element(&child.tag_name.to_ascii_lowercase(), component_map);
        if resolved == "img"
            && let Some(alt_text) = child.attribute_value("alt")
        {
            let normalized = normalize_text(alt_text);
            if !normalized.is_empty() {
                labels.push(normalized);
            }
        }

        labels.extend(collect_descendant_image_alt_text(child, component_map));
    }

    labels
}

fn native_label_for_control(
    node: &ParsedNode,
    wrapping_label: Option<&str>,
    context: &AnalysisContext,
) -> Option<String> {
    if let Some(id) = node.attribute_value("id")
        && let Some(label) = context.labels_by_control_id.get(id)
    {
        return Some(label.clone());
    }

    wrapping_label.map(str::to_string)
}

fn resolve_element(component: &str, component_map: &HashMap<String, String>) -> String {
    if let Some(mapped) = component_map.get(component) {
        return mapped.clone();
    }

    if matches!(
        component,
        "button"
            | "input"
            | "a"
            | "img"
            | "select"
            | "textarea"
            | "label"
            | "div"
            | "span"
            | "main"
            | "nav"
            | "header"
            | "footer"
            | "section"
            | "article"
            | "aside"
            | "form"
            | "p"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "ul"
            | "ol"
            | "li"
            | "table"
            | "tr"
            | "td"
            | "th"
            | "br"
            | "hr"
    ) {
        return component.to_string();
    }

    match component {
        "btn" | "iconbutton" | "closebutton" | "togglebutton" | "menubutton" => "button",
        "textfield" | "textinput" | "searchinput" => "input",
        "link" | "navlink" | "hyperlink" | "skiplink" => "a",
        "image" | "picture" | "avatar" | "thumbnail" => "img",
        "dropdown" | "combobox" | "multiselect" => "select",
        "textbox" | "richtext" => "textarea",
        "formlabel" | "fieldlabel" => "label",
        _ => component,
    }
    .to_string()
}

fn heading_level_from_tag(tag: &str) -> Option<u8> {
    match tag {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
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
    fn derives_link_name_from_child_image_alt_text() {
        let parsed =
            parse_html(r#"<a href="/reports"><img src="/icon.png" alt="View reports" /></a>"#)
                .expect("html should parse");

        let analyzed = analyze(&parsed);

        assert_eq!(analyzed[0].accessible_name.as_deref(), Some("View reports"));
    }

    #[test]
    fn derives_submit_input_name_from_default_value() {
        let parsed = parse_html(r#"<input type="submit" />"#).expect("html should parse");

        let analyzed = analyze(&parsed);

        assert_eq!(analyzed[0].accessible_name.as_deref(), Some("Submit"));
    }

    #[test]
    fn derives_image_input_name_from_alt_text() {
        let parsed =
            parse_html(r#"<input type="image" alt="Search" />"#).expect("html should parse");

        let analyzed = analyze(&parsed);

        assert_eq!(analyzed[0].accessible_name.as_deref(), Some("Search"));
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
