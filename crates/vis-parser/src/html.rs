use scraper::{ElementRef, Html, Node};

use crate::{Attribute, ParseError, ParsedNode};
use vis_diagnostics::Span;

pub fn parse_html(source: &str) -> Result<Vec<ParsedNode>, ParseError> {
    let document = Html::parse_document(source);
    let root = document.root_element();

    let has_explicit_html = source.to_ascii_lowercase().contains("<html");

    if has_explicit_html {
        let mut nodes = vec![convert_element(root, source)];
        annotate_spans(source, &mut nodes);
        return Ok(nodes);
    }

    let body = root
        .children()
        .filter_map(ElementRef::wrap)
        .find(|el| el.value().name.local.as_ref().eq_ignore_ascii_case("body"));

    let container = body.unwrap_or(root);
    let mut nodes = convert_children(container, source);
    annotate_spans(source, &mut nodes);
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

fn annotate_spans(source: &str, nodes: &mut [ParsedNode]) {
    let mut cursor = 0usize;
    for node in nodes {
        cursor = annotate_node(source, node, cursor);
    }
}

fn annotate_node(source: &str, node: &mut ParsedNode, cursor: usize) -> usize {
    if node.tag_name.is_empty() {
        return cursor;
    }

    let tag_lower = node.tag_name.as_str();
    let open_start = find_open_tag(source, tag_lower, cursor);
    let open_end = skip_tag_content(source, open_start + 1 + tag_lower.len());

    if is_self_closing(source, open_start, open_end) || is_void_element(tag_lower) {
        node.span = Span {
            start: open_start,
            end: open_end,
        };
        return open_end;
    }

    let mut pos = open_end;
    for child in &mut node.children {
        pos = annotate_node(source, child, pos);
    }

    let close_end = find_close_tag(source, tag_lower, pos);
    node.span = Span {
        start: open_start,
        end: close_end,
    };

    close_end
}

fn find_open_tag(source: &str, tag_name: &str, mut pos: usize) -> usize {
    let bytes = source.as_bytes();
    let len = bytes.len();

    while pos < len {
        if bytes[pos] == b'<' && pos + 1 < len {
            let after_lt = &source[pos + 1..];
            if after_lt
                .to_ascii_lowercase()
                .starts_with(&tag_name.to_ascii_lowercase())
            {
                let after_tag = &after_lt[tag_name.len()..];
                if after_tag.is_empty()
                    || after_tag.as_bytes()[0].is_ascii_whitespace()
                    || after_tag.as_bytes()[0] == b'>'
                    || after_tag.as_bytes()[0] == b'/'
                {
                    return pos;
                }
            }
        }
        pos += 1;
    }

    pos
}

fn skip_tag_content(source: &str, mut pos: usize) -> usize {
    let bytes = source.as_bytes();
    let len = bytes.len();

    while pos < len {
        match bytes[pos] {
            b'>' => return pos + 1,
            b'\'' | b'"' => {
                let quote = bytes[pos];
                pos += 1;
                while pos < len && bytes[pos] != quote {
                    pos += 1;
                }
                pos += 1;
            }
            _ => pos += 1,
        }
    }

    pos
}

fn is_self_closing(source: &str, _open_start: usize, open_end: usize) -> bool {
    open_end >= 3
        && &source.as_bytes()[open_end - 2..open_end] == b"/>"
        && !matches!(
            source.as_bytes().get(open_end - 3),
            Some(b'=' | b'"' | b'\'')
        )
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn find_close_tag(source: &str, tag_name: &str, mut pos: usize) -> usize {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let close_pattern = format!("</{}", tag_name);

    while pos < len {
        if bytes[pos] == b'<' && pos + 1 < len && bytes[pos + 1] == b'/' {
            let suffix = &source[pos..];
            if suffix
                .to_ascii_lowercase()
                .starts_with(&close_pattern.to_ascii_lowercase())
            {
                let after_tag = &suffix[close_pattern.len()..];
                if after_tag.is_empty()
                    || after_tag.as_bytes()[0].is_ascii_whitespace()
                    || after_tag.as_bytes()[0] == b'>'
                {
                    let mut close_end = pos + close_pattern.len();
                    while close_end < len && bytes[close_end] != b'>' {
                        close_end += 1;
                    }
                    return close_end + 1;
                }
            }
        }
        pos += 1;
    }

    len
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
