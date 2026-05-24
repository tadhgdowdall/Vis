use scraper::{ElementRef, Html, Node};

use crate::{Attribute, ParseError, ParsedNode};
use vis_diagnostics::Span;

pub fn parse_html(source: &str) -> Result<Vec<ParsedNode>, ParseError> {
    let document = Html::parse_document(source);
    let root = document.root_element();

    let has_explicit_html = source.to_ascii_lowercase().contains("<html");

    if has_explicit_html {
        return Ok(vec![convert_element(root, source)]);
    }

    let body = root
        .children()
        .filter_map(ElementRef::wrap)
        .find(|el| el.value().name.local.as_ref().eq_ignore_ascii_case("body"));

    let container = body.unwrap_or(root);
    let nodes = convert_children(container, source);
    Ok(nodes)
}

fn convert_children(parent: ElementRef, source: &str) -> Vec<ParsedNode> {
    let mut result = Vec::new();
    let mut text_buf = String::new();

    for child in parent.children() {
        if let Some(el) = ElementRef::wrap(child) {
            flush_text(&mut text_buf, &mut result, source);
            result.push(convert_element(el, source));
        } else if let Node::Text(text) = child.value() {
            text_buf.push_str(text);
        }
    }
    flush_text(&mut text_buf, &mut result, source);

    result
}

fn flush_text(text_buf: &mut String, result: &mut Vec<ParsedNode>, source: &str) {
    let trimmed = normalize_whitespace(text_buf);
    if !trimmed.is_empty() {
        result.push(ParsedNode {
            tag_name: String::new(),
            attributes: Vec::new(),
            text_content: trimmed,
            span: Span {
                start: 0,
                end: source.len(),
            },
            children: Vec::new(),
        });
        text_buf.clear();
    }
}

fn convert_element(el: ElementRef, source: &str) -> ParsedNode {
    let tag_name = el.value().name.local.as_ref().to_ascii_lowercase();

    let attributes: Vec<Attribute> = el
        .value()
        .attrs
        .iter()
        .map(|(name, value)| Attribute {
            name: name.local.as_ref().to_string(),
            value: Some(value.to_string()),
            span: Span { start: 0, end: 0 },
        })
        .collect();

    let children = convert_children(el, source);

    ParsedNode {
        tag_name,
        attributes,
        text_content: String::new(),
        span: Span {
            start: 0,
            end: source.len(),
        },
        children,
    }
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
