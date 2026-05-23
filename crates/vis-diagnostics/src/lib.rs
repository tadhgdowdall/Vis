mod diagnostic;
mod render;

pub use diagnostic::{Diagnostic, Position, Span};
pub use render::{position_for_offset, render_diagnostic};
