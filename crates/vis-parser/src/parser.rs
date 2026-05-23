use crate::{
    ast::{Attribute, ParsedNode},
    error::ParseError,
};
use vis_diagnostics::Span;

struct OpenNode {
    node: ParsedNode,
}

pub fn parse_document(source: &str) -> Result<Vec<ParsedNode>, ParseError> {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut stack: Vec<OpenNode> = Vec::new();
    let mut roots = Vec::new();

    while index < bytes.len() {
        if bytes[index] == b'<' {
            if source[index..].starts_with("<!--") {
                let Some(end) = source[index + 4..].find("-->") else {
                    return Err(ParseError::new("unterminated HTML comment"));
                };
                index += 4 + end + 3;
                continue;
            }

            if index + 1 < bytes.len() && bytes[index + 1] == b'/' {
                let (tag_name, end_index) = parse_closing_tag(source, index)?;
                close_node(&mut stack, &mut roots, &tag_name, end_index);
                index = end_index;
                continue;
            }

            let (node, end_index, self_closing) = parse_open_tag(source, index)?;

            if self_closing || is_void_element(&node.tag_name) {
                attach_node(&mut stack, &mut roots, node);
            } else {
                stack.push(OpenNode { node });
            }

            index = end_index;
        } else {
            let next_tag = source[index..]
                .find('<')
                .map(|offset| index + offset)
                .unwrap_or(source.len());
            let text = &source[index..next_tag];

            if let Some(open_node) = stack.last_mut() {
                open_node.node.text_content.push_str(text);
            }

            index = next_tag;
        }
    }

    while let Some(mut open_node) = stack.pop() {
        open_node.node.span.end = source.len();
        attach_node(&mut stack, &mut roots, open_node.node);
    }

    Ok(roots)
}

fn parse_open_tag(
    source: &str,
    start_index: usize,
) -> Result<(ParsedNode, usize, bool), ParseError> {
    let bytes = source.as_bytes();
    let mut index = start_index + 1;
    skip_whitespace(bytes, &mut index);
    let tag_start = index;

    while index < bytes.len() && is_tag_name_char(bytes[index] as char) {
        index += 1;
    }

    if tag_start == index {
        return Err(ParseError::new(format!(
            "expected tag name at byte {start_index}"
        )));
    }

    let tag_name = source[tag_start..index].to_ascii_lowercase();
    let mut attributes = Vec::new();
    let mut self_closing = false;

    loop {
        skip_whitespace(bytes, &mut index);

        if index >= bytes.len() {
            return Err(ParseError::new(format!("unterminated tag <{tag_name}>")));
        }

        match bytes[index] {
            b'>' => {
                index += 1;
                break;
            }
            b'/' => {
                self_closing = true;
                index += 1;
                if index < bytes.len() && bytes[index] == b'>' {
                    index += 1;
                    break;
                }
            }
            _ => {
                let (attribute, next_index) = parse_attribute(source, index)?;
                attributes.push(attribute);
                index = next_index;
            }
        }
    }

    Ok((
        ParsedNode {
            tag_name,
            attributes,
            text_content: String::new(),
            span: Span {
                start: start_index,
                end: index,
            },
            children: Vec::new(),
        },
        index,
        self_closing,
    ))
}

fn parse_attribute(source: &str, start_index: usize) -> Result<(Attribute, usize), ParseError> {
    let bytes = source.as_bytes();
    let mut index = start_index;
    let name_start = index;

    while index < bytes.len() && is_attribute_name_char(bytes[index] as char) {
        index += 1;
    }

    if name_start == index {
        return Err(ParseError::new(format!(
            "expected attribute name at byte {start_index}"
        )));
    }

    let name = source[name_start..index].to_string();
    skip_whitespace(bytes, &mut index);
    let mut value = None;

    if index < bytes.len() && bytes[index] == b'=' {
        index += 1;
        skip_whitespace(bytes, &mut index);
        let (parsed_value, next_index) = parse_attribute_value(source, index)?;
        value = Some(parsed_value);
        index = next_index;
    }

    Ok((
        Attribute {
            name,
            value,
            span: Span {
                start: start_index,
                end: index,
            },
        },
        index,
    ))
}

fn parse_attribute_value(source: &str, start_index: usize) -> Result<(String, usize), ParseError> {
    let bytes = source.as_bytes();

    if start_index >= bytes.len() {
        return Err(ParseError::new("expected attribute value"));
    }

    match bytes[start_index] {
        b'"' | b'\'' => {
            let quote = bytes[start_index];
            let mut index = start_index + 1;
            while index < bytes.len() && bytes[index] != quote {
                index += 1;
            }

            if index >= bytes.len() {
                return Err(ParseError::new("unterminated quoted attribute value"));
            }

            Ok((source[start_index + 1..index].to_string(), index + 1))
        }
        b'{' => {
            let mut depth = 1;
            let mut index = start_index + 1;

            while index < bytes.len() && depth > 0 {
                match bytes[index] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                index += 1;
            }

            if depth != 0 {
                return Err(ParseError::new("unterminated JSX attribute expression"));
            }

            Ok((source[start_index..index].to_string(), index))
        }
        _ => {
            let mut index = start_index;
            while index < bytes.len()
                && !matches!(bytes[index], b'>' | b'/')
                && !(bytes[index] as char).is_whitespace()
            {
                index += 1;
            }

            Ok((source[start_index..index].to_string(), index))
        }
    }
}

fn parse_closing_tag(source: &str, start_index: usize) -> Result<(String, usize), ParseError> {
    let bytes = source.as_bytes();
    let mut index = start_index + 2;
    skip_whitespace(bytes, &mut index);
    let name_start = index;

    while index < bytes.len() && is_tag_name_char(bytes[index] as char) {
        index += 1;
    }

    if name_start == index {
        return Err(ParseError::new(format!(
            "expected closing tag name at byte {start_index}"
        )));
    }

    let tag_name = source[name_start..index].to_ascii_lowercase();

    while index < bytes.len() && bytes[index] != b'>' {
        index += 1;
    }

    if index >= bytes.len() {
        return Err(ParseError::new(format!(
            "unterminated closing tag </{tag_name}>"
        )));
    }

    Ok((tag_name, index + 1))
}

fn close_node(
    stack: &mut Vec<OpenNode>,
    roots: &mut Vec<ParsedNode>,
    tag_name: &str,
    end_index: usize,
) {
    while let Some(mut open_node) = stack.pop() {
        let matched = open_node.node.tag_name == tag_name;
        open_node.node.span.end = end_index;
        attach_node(stack, roots, open_node.node);

        if matched {
            break;
        }
    }
}

fn attach_node(stack: &mut [OpenNode], roots: &mut Vec<ParsedNode>, node: ParsedNode) {
    if let Some(parent) = stack.last_mut() {
        parent.node.children.push(node);
    } else {
        roots.push(node);
    }
}

fn skip_whitespace(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() && (bytes[*index] as char).is_whitespace() {
        *index += 1;
    }
}

fn is_tag_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':')
}

fn is_attribute_name_char(ch: char) -> bool {
    is_tag_name_char(ch)
}

fn is_void_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
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
