use vis_ir::A11yNode;
use vis_parser::ParsedNode;

pub fn analyze(nodes: &[ParsedNode]) -> Vec<A11yNode> {
    nodes.iter().map(analyze_node).collect()
}

fn analyze_node(node: &ParsedNode) -> A11yNode {
    let tag_name = node.tag_name.to_ascii_lowercase();
    let has_click_handler = node
        .attributes
        .iter()
        .any(|attribute| attribute.name.eq_ignore_ascii_case("onclick"));
    let href = node.attribute_value("href");
    let tab_index = node.attribute_value("tabindex");
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

    let accessible_name = aria_label
        .filter(|label| !label.is_empty())
        .or_else(|| match tag_name.as_str() {
            "img" => alt_text.clone(),
            "button" => (!text_content.is_empty()).then_some(text_content.clone()),
            _ => None,
        });

    A11yNode {
        tag_name,
        interactive,
        focusable,
        accessible_name,
        has_click_handler,
        alt_text,
        span: node.span,
        children: node.children.iter().map(analyze_node).collect(),
    }
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
