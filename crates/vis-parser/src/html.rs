use crate::{ParseError, ParsedNode, parser::parse_document};

pub fn parse_html(source: &str) -> Result<Vec<ParsedNode>, ParseError> {
    parse_document(source)
}
