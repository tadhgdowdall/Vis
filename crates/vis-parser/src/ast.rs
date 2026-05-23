use vis_diagnostics::Span;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ParsedNode {
    pub tag_name: String,
    pub attributes: Vec<Attribute>,
    pub text_content: String,
    pub span: Span,
    pub children: Vec<ParsedNode>,
}

impl ParsedNode {
    pub fn attribute_value(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name.eq_ignore_ascii_case(name))
            .and_then(|attribute| attribute.value.as_deref())
    }

    pub fn text(&self) -> String {
        let mut text = self.text_content.clone();

        for child in &self.children {
            let child_text = child.text();
            if !text.trim().is_empty() && !child_text.is_empty() {
                text.push(' ');
            }

            text.push_str(child_text.trim());
        }

        text.trim().to_string()
    }
}
