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
    pub span: Span,
    pub children: Vec<A11yNode>,
}
