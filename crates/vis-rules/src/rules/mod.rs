pub mod button_label;
pub mod clickable_div;
pub mod form_control_label;
pub mod img_alt;
pub mod link_semantics;

use crate::RuleFn;

pub const ALL: &[RuleFn] = &[
    img_alt::check,
    link_semantics::check,
    clickable_div::check,
    button_label::check,
    form_control_label::check,
];
