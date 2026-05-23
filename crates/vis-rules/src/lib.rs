pub mod rules;

use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

type RuleFn = fn(&str, &A11yNode, &mut Vec<Diagnostic>);

pub fn run_all(file_path: &str, nodes: &[A11yNode]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for node in nodes {
        for rule in rules::ALL {
            rule(file_path, node, &mut diagnostics);
        }
    }

    diagnostics
}
