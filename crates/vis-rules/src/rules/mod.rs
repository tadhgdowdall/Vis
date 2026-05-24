pub mod autocomplete;
pub mod button_label;
pub mod clickable_div;
pub mod empty_heading;
pub mod form_control_label;
pub mod form_no_submit;
pub mod heading_hierarchy;
pub mod html_lang;
pub mod img_alt;
pub mod link_label;
pub mod link_semantics;
pub mod page_title;
pub mod placeholder_label;
pub mod tabindex_misuse;
pub mod title_label;

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
    empty_heading::check,
    placeholder_label::check,
    form_no_submit::check,
    title_label::check,
];
