mod ast;
mod error;
mod html;
mod jsx;
mod parser;

pub use ast::{Attribute, ParsedNode};
pub use error::ParseError;
pub use html::parse_html;
pub use jsx::parse_jsx;
