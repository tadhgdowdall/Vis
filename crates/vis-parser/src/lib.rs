mod ast;
mod error;
mod html;
mod parser;
mod tsx;

pub use ast::{Attribute, ParsedNode};
pub use error::ParseError;
pub use html::parse_html;
pub use tsx::parse_tsx as parse_jsx;
