use swc_common::{FileName, SourceMap, Spanned, sync::Lrc};
use swc_ecma_ast::{
    EsVersion, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXElement, JSXElementChild,
    JSXElementName, JSXExpr, JSXFragment, JSXText, Str,
};
use swc_ecma_parser::{Syntax, TsSyntax, parse_file_as_module};
use swc_ecma_visit::{Visit, VisitWith};

use crate::{Attribute, ParseError, ParsedNode};
use vis_diagnostics::Span;

pub fn parse_tsx(source: &str) -> Result<Vec<ParsedNode>, ParseError> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom("input.tsx".into())),
        source.to_string(),
    );
    let mut recovered_errors = Vec::new();

    let module = parse_file_as_module(
        &fm,
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        }),
        EsVersion::latest(),
        None,
        &mut recovered_errors,
    )
    .map_err(|error| ParseError::new(error.kind().msg().to_string()))?;

    if !recovered_errors.is_empty() {
        let messages: Vec<String> = recovered_errors
            .into_iter()
            .map(|e| e.kind().msg().to_string())
            .collect();
        return Err(ParseError::new(messages.join("; ")));
    }

    let mut collector = JsxCollector {
        roots: Vec::new(),
        source,
    };

    module.visit_with(&mut collector);
    Ok(collector.roots)
}

struct JsxCollector<'src> {
    roots: Vec<ParsedNode>,
    source: &'src str,
}

impl Visit for JsxCollector<'_> {
    fn visit_jsx_element(&mut self, node: &JSXElement) {
        self.roots.push(convert_jsx_element(node, self.source));
    }

    fn visit_jsx_fragment(&mut self, node: &JSXFragment) {
        self.roots.push(convert_jsx_fragment(node, self.source));
    }
}

struct NestedJsxCollector<'src> {
    nodes: Vec<ParsedNode>,
    source: &'src str,
}

impl Visit for NestedJsxCollector<'_> {
    fn visit_jsx_element(&mut self, node: &JSXElement) {
        self.nodes.push(convert_jsx_element(node, self.source));
    }

    fn visit_jsx_fragment(&mut self, node: &JSXFragment) {
        self.nodes.push(convert_jsx_fragment(node, self.source));
    }
}

fn convert_jsx_element(node: &JSXElement, source: &str) -> ParsedNode {
    let tag_name = jsx_element_name_to_string(&node.opening.name);
    let attributes = node
        .opening
        .attrs
        .iter()
        .filter_map(|attr| {
            if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                Some(convert_jsx_attr(attr, source))
            } else {
                None
            }
        })
        .collect();

    let children: Vec<ParsedNode> = node
        .children
        .iter()
        .filter_map(|child| convert_jsx_child(child, source))
        .collect();

    let text_content = children
        .iter()
        .map(|child| child.text_content.clone())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    ParsedNode {
        tag_name,
        attributes,
        text_content,
        span: convert_span(node.span),
        children,
    }
}

fn convert_jsx_fragment(node: &JSXFragment, source: &str) -> ParsedNode {
    let children: Vec<ParsedNode> = node
        .children
        .iter()
        .filter_map(|child| convert_jsx_child(child, source))
        .collect();

    let text_content = children
        .iter()
        .map(|child| child.text_content.clone())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    ParsedNode {
        tag_name: String::new(),
        attributes: Vec::new(),
        text_content,
        span: convert_span(node.span),
        children,
    }
}

fn convert_jsx_child(child: &JSXElementChild, source: &str) -> Option<ParsedNode> {
    match child {
        JSXElementChild::JSXElement(element) => Some(convert_jsx_element(element, source)),
        JSXElementChild::JSXFragment(fragment) => Some(convert_jsx_fragment(fragment, source)),
        JSXElementChild::JSXText(text) => Some(convert_jsx_text(text)),
        JSXElementChild::JSXExprContainer(expr) => match &expr.expr {
            JSXExpr::Expr(expression) => {
                let (start, end) = span_to_offsets(expression.span());
                let snippet = source.get(start..end).unwrap_or("").to_string();

                let mut nested = NestedJsxCollector {
                    nodes: Vec::new(),
                    source,
                };
                expression.as_ref().visit_with(&mut nested);

                Some(ParsedNode {
                    tag_name: String::new(),
                    attributes: Vec::new(),
                    text_content: snippet,
                    span: convert_span(expression.span()),
                    children: nested.nodes,
                })
            }
            JSXExpr::JSXEmptyExpr(_) => None,
        },
        JSXElementChild::JSXSpreadChild(_) => None,
    }
}

fn convert_jsx_text(text: &JSXText) -> ParsedNode {
    let content = text.value.to_string();
    let normalized = content
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    ParsedNode {
        tag_name: String::new(),
        attributes: Vec::new(),
        text_content: normalized,
        span: convert_span(text.span),
        children: Vec::new(),
    }
}

fn convert_jsx_attr(attr: &swc_ecma_ast::JSXAttr, source: &str) -> Attribute {
    let name = match &attr.name {
        JSXAttrName::Ident(ident) => ident.sym.to_string(),
        JSXAttrName::JSXNamespacedName(ns) => {
            format!("{}:{}", ns.ns.sym, ns.name.sym)
        }
    };

    let value = attr.value.as_ref().map(|value| match value {
        JSXAttrValue::Str(Str { value, .. }) => value.to_string_lossy().to_string(),
        JSXAttrValue::JSXExprContainer(expr) => match &expr.expr {
            JSXExpr::Expr(expression) => {
                let (start, end) = span_to_offsets(expression.span());
                source.get(start..end).unwrap_or("").to_string()
            }
            JSXExpr::JSXEmptyExpr(_) => String::new(),
        },
        JSXAttrValue::JSXElement(element) => {
            let (start, end) = span_to_offsets(element.span);
            source.get(start..end).unwrap_or("").to_string()
        }
        JSXAttrValue::JSXFragment(fragment) => {
            let (start, end) = span_to_offsets(fragment.span);
            source.get(start..end).unwrap_or("").to_string()
        }
    });

    let span = convert_span(attr.span);

    Attribute { name, value, span }
}

fn jsx_element_name_to_string(name: &JSXElementName) -> String {
    match name {
        JSXElementName::Ident(ident) => ident.sym.to_string(),
        JSXElementName::JSXMemberExpr(member) => {
            let mut parts = vec![member.prop.sym.to_string()];
            let mut current = &member.obj;

            loop {
                match current {
                    swc_ecma_ast::JSXObject::Ident(ident) => {
                        parts.push(ident.sym.to_string());
                        break;
                    }
                    swc_ecma_ast::JSXObject::JSXMemberExpr(member) => {
                        parts.push(member.prop.sym.to_string());
                        current = &member.obj;
                    }
                }
            }

            parts.reverse();
            parts.join(".")
        }
        JSXElementName::JSXNamespacedName(ns) => {
            format!("{}:{}", ns.ns.sym, ns.name.sym)
        }
    }
}

fn convert_span(span: swc_common::Span) -> Span {
    Span {
        start: (span.lo.0 as usize).saturating_sub(1),
        end: (span.hi.0 as usize).saturating_sub(1),
    }
}

fn span_to_offsets(span: swc_common::Span) -> (usize, usize) {
    (
        (span.lo.0 as usize).saturating_sub(1),
        (span.hi.0 as usize).saturating_sub(1),
    )
}

#[cfg(test)]
mod tests {
    use super::parse_tsx;

    #[test]
    fn parses_simple_jsx_element() {
        let source = r#"<button onClick={save}>Save</button>"#;
        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].tag_name, "button");
        assert_eq!(nodes[0].text_content, "Save");
        assert_eq!(nodes[0].attributes.len(), 1);
        assert_eq!(nodes[0].attributes[0].name, "onClick");
    }

    #[test]
    fn parses_jsx_in_function() {
        let source = r#"
            export function App() {
                return <main><img src="/hero.png" alt="Hero" /></main>;
            }
        "#;

        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].tag_name, "main");
        assert_eq!(nodes[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].tag_name, "img");
        assert_eq!(nodes[0].children[0].attribute_value("alt"), Some("Hero"));
    }

    #[test]
    fn parses_jsx_expression_container_as_text() {
        let source = r#"<div>{user.name}</div>"#;
        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].tag_name, "div");
        assert_eq!(nodes[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].text_content, "user.name");
    }

    #[test]
    fn parses_jsx_fragment() {
        let source = r#"<><span>hello</span></>"#;
        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].tag_name, "");
        assert_eq!(nodes[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].tag_name, "span");
    }

    #[test]
    fn parses_self_closing_element() {
        let source = r#"<input type="text" />"#;
        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].tag_name, "input");
        assert_eq!(nodes[0].attribute_value("type"), Some("text"));
    }

    #[test]
    fn parses_boolean_attribute() {
        let source = r#"<button disabled>Save</button>"#;
        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes[0].attributes.len(), 1);
        assert_eq!(nodes[0].attributes[0].name, "disabled");
        assert_eq!(nodes[0].attributes[0].value, None);
    }

    #[test]
    fn parses_member_expression_component() {
        let source = r#"<UI.Button>Click</UI.Button>"#;
        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes[0].tag_name, "UI.Button");
    }

    #[test]
    fn extracts_expression_attribute_value() {
        let source = r#"<div className={styles.container} />"#;
        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(
            nodes[0].attribute_value("className"),
            Some("styles.container")
        );
    }

    #[test]
    fn preserves_href_attribute() {
        let source = r#"<a href="/reports">View reports</a>"#;
        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes[0].tag_name, "a");
        assert_eq!(nodes[0].attribute_value("href"), Some("/reports"));
    }

    #[test]
    fn parses_multiple_root_jsx_trees() {
        let source = r#"
            const a = <div>1</div>;
            const b = <span>2</span>;
        "#;

        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].tag_name, "div");
        assert_eq!(nodes[1].tag_name, "span");
    }

    #[test]
    fn parses_jsx_inside_conditional_expression() {
        let source = r#"<div>{show && <span>visible</span>}</div>"#;
        let nodes = parse_tsx(source).expect("tsx should parse");

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].tag_name, "div");
        assert_eq!(nodes[0].children.len(), 1);
        assert!(nodes[0].children[0].text_content.contains("span"));
        assert_eq!(nodes[0].children[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].children[0].tag_name, "span");
    }
}
