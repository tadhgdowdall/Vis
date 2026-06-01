pub mod aria_data;
pub mod autocomplete;
pub mod button_label;
pub mod clickable_div;
pub mod empty_heading;
pub mod form_control_label;
pub mod form_no_submit;
pub mod heading_hierarchy;
pub mod html_lang;
pub mod img_alt;
pub mod invalid_aria;
pub mod invalid_role;
pub mod keyboard_handler;
pub mod link_label;
pub mod link_semantics;
pub mod page_title;
pub mod placeholder_label;
pub mod redundant_role;
pub mod tabindex_misuse;
pub mod title_label;

use crate::Rule;
use vis_diagnostics::Severity;

pub(crate) const ALL: &[Rule] = &[
    Rule {
        code: "a11y::missing_alt",
        default_severity: Severity::Error,
        check: img_alt::check,
    },
    Rule {
        code: "a11y::heading_hierarchy",
        default_severity: Severity::Error,
        check: heading_hierarchy::check,
    },
    Rule {
        code: "a11y::html_lang",
        default_severity: Severity::Error,
        check: html_lang::check,
    },
    Rule {
        code: "a11y::page_title",
        default_severity: Severity::Error,
        check: page_title::check,
    },
    Rule {
        code: "a11y::link_label",
        default_severity: Severity::Error,
        check: link_label::check,
    },
    Rule {
        code: "a11y::link_semantics",
        default_severity: Severity::Error,
        check: link_semantics::check,
    },
    Rule {
        code: "a11y::tabindex_misuse",
        default_severity: Severity::Error,
        check: tabindex_misuse::check,
    },
    Rule {
        code: "a11y::clickable_div",
        default_severity: Severity::Error,
        check: clickable_div::check,
    },
    Rule {
        code: "a11y::button_label",
        default_severity: Severity::Error,
        check: button_label::check,
    },
    Rule {
        code: "a11y::form_control_label",
        default_severity: Severity::Error,
        check: form_control_label::check,
    },
    Rule {
        code: "a11y::autocomplete",
        default_severity: Severity::Off,
        check: autocomplete::check,
    },
    Rule {
        code: "a11y::empty_heading",
        default_severity: Severity::Warning,
        check: empty_heading::check,
    },
    Rule {
        code: "a11y::placeholder_label",
        default_severity: Severity::Warning,
        check: placeholder_label::check,
    },
    Rule {
        code: "a11y::form_no_submit",
        default_severity: Severity::Warning,
        check: form_no_submit::check,
    },
    Rule {
        code: "a11y::title_label",
        default_severity: Severity::Warning,
        check: title_label::check,
    },
    Rule {
        code: "a11y::invalid_aria",
        default_severity: Severity::Error,
        check: invalid_aria::check,
    },
    Rule {
        code: "a11y::invalid_role",
        default_severity: Severity::Error,
        check: invalid_role::check,
    },
    Rule {
        code: "a11y::redundant_role",
        default_severity: Severity::Warning,
        check: redundant_role::check,
    },
    Rule {
        code: "a11y::keyboard_handler",
        default_severity: Severity::Warning,
        check: keyboard_handler::check,
    },
];
