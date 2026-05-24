use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

fn valid_aria_attrs() -> &'static BTreeSet<&'static str> {
    static ATTRS: OnceLock<BTreeSet<&str>> = OnceLock::new();
    ATTRS.get_or_init(|| {
        [
            "aria-activedescendant",
            "aria-atomic",
            "aria-autocomplete",
            "aria-braillelabel",
            "aria-brailleroledescription",
            "aria-busy",
            "aria-checked",
            "aria-colcount",
            "aria-colindex",
            "aria-colindextext",
            "aria-colspan",
            "aria-controls",
            "aria-current",
            "aria-describedby",
            "aria-description",
            "aria-details",
            "aria-disabled",
            "aria-dropeffect",
            "aria-errormessage",
            "aria-expanded",
            "aria-flowto",
            "aria-grabbed",
            "aria-haspopup",
            "aria-hidden",
            "aria-invalid",
            "aria-keyshortcuts",
            "aria-label",
            "aria-labelledby",
            "aria-level",
            "aria-live",
            "aria-modal",
            "aria-multiline",
            "aria-multiselectable",
            "aria-orientation",
            "aria-owns",
            "aria-placeholder",
            "aria-posinset",
            "aria-pressed",
            "aria-readonly",
            "aria-relevant",
            "aria-required",
            "aria-roledescription",
            "aria-rowcount",
            "aria-rowindex",
            "aria-rowindextext",
            "aria-rowspan",
            "aria-selected",
            "aria-setsize",
            "aria-sort",
            "aria-valuemax",
            "aria-valuemin",
            "aria-valuenow",
            "aria-valuetext",
        ]
        .iter()
        .copied()
        .collect()
    })
}

fn valid_roles() -> &'static BTreeSet<&'static str> {
    static ROLES: OnceLock<BTreeSet<&str>> = OnceLock::new();
    ROLES.get_or_init(|| {
        [
            "alert",
            "alertdialog",
            "application",
            "article",
            "banner",
            "blockquote",
            "button",
            "caption",
            "cell",
            "checkbox",
            "code",
            "columnheader",
            "combobox",
            "command",
            "comment",
            "complementary",
            "composite",
            "contentinfo",
            "definition",
            "deletion",
            "dialog",
            "directory",
            "document",
            "emphasis",
            "feed",
            "figure",
            "form",
            "generic",
            "grid",
            "gridcell",
            "group",
            "heading",
            "img",
            "input",
            "insertion",
            "landmark",
            "link",
            "list",
            "listbox",
            "listitem",
            "log",
            "main",
            "mark",
            "marquee",
            "math",
            "menu",
            "menubar",
            "menuitem",
            "menuitemcheckbox",
            "menuitemradio",
            "meter",
            "navigation",
            "none",
            "note",
            "option",
            "paragraph",
            "presentation",
            "progressbar",
            "radio",
            "radiogroup",
            "range",
            "region",
            "row",
            "rowgroup",
            "rowheader",
            "scrollbar",
            "search",
            "searchbox",
            "section",
            "sectionhead",
            "select",
            "separator",
            "slider",
            "spinbutton",
            "status",
            "strong",
            "structure",
            "subscript",
            "suggestion",
            "superscript",
            "switch",
            "tab",
            "table",
            "tablist",
            "tabpanel",
            "term",
            "textbox",
            "time",
            "timer",
            "toolbar",
            "tooltip",
            "tree",
            "treegrid",
            "treeitem",
            "widget",
            "window",
        ]
        .iter()
        .copied()
        .collect()
    })
}

fn native_roles() -> &'static HashMap<&'static str, &'static [&'static str]> {
    static MAP: OnceLock<HashMap<&str, &[&str]>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("a", &["link"][..]);
        m.insert("article", &["article"][..]);
        m.insert("aside", &["complementary"][..]);
        m.insert("button", &["button"][..]);
        m.insert("dialog", &["dialog"][..]);
        m.insert("footer", &["contentinfo"][..]);
        m.insert("form", &["form"][..]);
        m.insert("h1", &["heading"][..]);
        m.insert("h2", &["heading"][..]);
        m.insert("h3", &["heading"][..]);
        m.insert("h4", &["heading"][..]);
        m.insert("h5", &["heading"][..]);
        m.insert("h6", &["heading"][..]);
        m.insert("header", &["banner"][..]);
        m.insert("hr", &["separator"][..]);
        m.insert("img", &["img"][..]);
        m.insert("input", &["textbox"][..]);
        m.insert("li", &["listitem"][..]);
        m.insert("main", &["main"][..]);
        m.insert("nav", &["navigation"][..]);
        m.insert("ol", &["list"][..]);
        m.insert("progress", &["progressbar"][..]);
        m.insert("section", &["region"][..]);
        m.insert("select", &["combobox"][..]);
        m.insert("table", &["table"][..]);
        m.insert("td", &["cell"][..]);
        m.insert("textarea", &["textbox"][..]);
        m.insert("th", &["columnheader"][..]);
        m.insert("tr", &["row"][..]);
        m.insert("ul", &["list"][..]);
        m
    })
}

pub fn is_valid_aria_attr(name: &str) -> bool {
    valid_aria_attrs().contains(name.to_ascii_lowercase().as_str())
}

pub fn is_valid_role(role: &str) -> bool {
    valid_roles().contains(role.to_ascii_lowercase().as_str())
}

pub fn get_native_roles(tag: &str) -> Option<&'static [&'static str]> {
    native_roles().get(tag).copied()
}
