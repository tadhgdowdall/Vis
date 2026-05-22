use vis_diagnostics::Span;

pub struct A11yNode {
    pub tag_name: String,
    pub interactive: bool,
    pub focusable: bool,
    pub accessible_name: Option<String>,
    pub has_click_handler: bool,
    pub span: Span,
    pub children: Vec<A11yNode>,
}
