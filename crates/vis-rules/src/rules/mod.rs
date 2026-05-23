pub mod button_label;
pub mod clickable_div;
pub mod img_alt;

use crate::RuleFn;

pub const ALL: &[RuleFn] = &[img_alt::check, clickable_div::check, button_label::check];
