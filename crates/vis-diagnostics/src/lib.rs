mod diagnostic;
mod json;
mod render;

pub use diagnostic::{Diagnostic, Position, Severity, Span};
pub use json::render_diagnostics_json;
pub use render::{position_for_offset, render_diagnostic};
