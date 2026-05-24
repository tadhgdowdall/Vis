use vis_diagnostics::Span;

#[derive(Clone, Debug)]
pub struct A11yNode {
    pub tag_name: String,
    pub resolved_tag_name: String,
    pub href: Option<String>,
    pub input_type: Option<String>,
    pub interactive: bool,
    pub focusable: bool,
    pub accessible_name: Option<String>,
    pub has_click_handler: bool,
    pub alt_text: Option<String>,
    pub tab_index: Option<String>,
    pub heading_level: Option<u8>,
    pub lang: Option<String>,
    pub autocomplete: Option<String>,
    pub placeholder: Option<String>,
    pub title_attr: Option<String>,
    pub has_submit_handler: bool,
    pub role: Option<String>,
    pub aria_attrs: Vec<String>,
    pub aria_hidden: bool,
    pub suppression_codes: Vec<String>,
    pub span: Span,
    pub children: Vec<A11yNode>,
}
