use crate::{ParsedNode, parse_like_html};

pub fn parse_jsx(source: &str) -> Result<Vec<ParsedNode>, String> {
    parse_like_html(source)
}
