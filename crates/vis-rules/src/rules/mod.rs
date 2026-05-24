pub mod autocomplete;
pub mod button_label;
pub mod clickable_div;
pub mod form_control_label;
pub mod heading_hierarchy;
pub mod html_lang;
pub mod img_alt;
pub mod link_label;
pub mod link_semantics;
pub mod page_title;
pub mod tabindex_misuse;

use crate::RuleFn;

pub const ALL: &[RuleFn] = &[
    img_alt::check,
    heading_hierarchy::check,
    html_lang::check,
    page_title::check,
    link_label::check,
    link_semantics::check,
    tabindex_misuse::check,
    clickable_div::check,
    button_label::check,
    form_control_label::check,
    autocomplete::check,
];