mod html;
mod jsx;

use vis_diagnostics::Span;

pub use html::parse_html;
pub use jsx::parse_jsx;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ParsedNode {
    pub tag_name: String,
    pub attributes: Vec<Attribute>,
    pub text_content: String,
    pub span: Span,
    pub children: Vec<ParsedNode>,
}

impl ParsedNode {
    pub fn attribute_value(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name.eq_ignore_ascii_case(name))
            .and_then(|attribute| attribute.value.as_deref())
    }

    pub fn text(&self) -> String {
        let mut text = self.text_content.clone();

        for child in &self.children {
            let child_text = child.text();
            if !text.trim().is_empty() && !child_text.is_empty() {
                text.push(' ');
            }

            text.push_str(child_text.trim());
        }

        text.trim().to_string()
    }
}

pub fn parse_like_html(source: &str) -> Result<Vec<ParsedNode>, String> {
    parser::parse_document(source)
}

mod parser {
    use super::{Attribute, ParsedNode};
    use vis_diagnostics::Span;

    struct OpenNode {
        node: ParsedNode,
    }

    pub fn parse_document(source: &str) -> Result<Vec<ParsedNode>, String> {
        let bytes = source.as_bytes();
        let mut index = 0;
        let mut stack: Vec<OpenNode> = Vec::new();
        let mut roots = Vec::new();

        while index < bytes.len() {
            if bytes[index] == b'<' {
                if source[index..].starts_with("<!--") {
                    let Some(end) = source[index + 4..].find("-->") else {
                        return Err("unterminated HTML comment".to_string());
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
    ) -> Result<(ParsedNode, usize, bool), String> {
        let bytes = source.as_bytes();
        let mut index = start_index + 1;
        skip_whitespace(bytes, &mut index);
        let tag_start = index;

        while index < bytes.len() && is_tag_name_char(bytes[index] as char) {
            index += 1;
        }

        if tag_start == index {
            return Err(format!("expected tag name at byte {}", start_index));
        }

        let tag_name = source[tag_start..index].to_ascii_lowercase();
        let mut attributes = Vec::new();
        let mut self_closing = false;

        loop {
            skip_whitespace(bytes, &mut index);

            if index >= bytes.len() {
                return Err(format!("unterminated tag <{}>", tag_name));
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

    fn parse_attribute(source: &str, start_index: usize) -> Result<(Attribute, usize), String> {
        let bytes = source.as_bytes();
        let mut index = start_index;
        let name_start = index;

        while index < bytes.len() && is_attribute_name_char(bytes[index] as char) {
            index += 1;
        }

        if name_start == index {
            return Err(format!("expected attribute name at byte {}", start_index));
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

    fn parse_attribute_value(source: &str, start_index: usize) -> Result<(String, usize), String> {
        let bytes = source.as_bytes();

        if start_index >= bytes.len() {
            return Err("expected attribute value".to_string());
        }

        match bytes[start_index] {
            b'"' | b'\'' => {
                let quote = bytes[start_index];
                let mut index = start_index + 1;
                while index < bytes.len() && bytes[index] != quote {
                    index += 1;
                }

                if index >= bytes.len() {
                    return Err("unterminated quoted attribute value".to_string());
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
                    return Err("unterminated JSX attribute expression".to_string());
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

    fn parse_closing_tag(source: &str, start_index: usize) -> Result<(String, usize), String> {
        let bytes = source.as_bytes();
        let mut index = start_index + 2;
        skip_whitespace(bytes, &mut index);
        let name_start = index;

        while index < bytes.len() && is_tag_name_char(bytes[index] as char) {
            index += 1;
        }

        if name_start == index {
            return Err(format!("expected closing tag name at byte {}", start_index));
        }

        let tag_name = source[name_start..index].to_ascii_lowercase();

        while index < bytes.len() && bytes[index] != b'>' {
            index += 1;
        }

        if index >= bytes.len() {
            return Err(format!("unterminated closing tag </{}>", tag_name));
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
}
