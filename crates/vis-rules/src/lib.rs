pub mod rules;

use vis_diagnostics::Diagnostic;
use vis_ir::A11yNode;

pub fn run_all(file_path: &str, nodes: &[A11yNode]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for node in nodes {
        rules::img_alt::check(file_path, node, &mut diagnostics);
        rules::clickable_div::check(file_path, node, &mut diagnostics);
        rules::button_label::check(file_path, node, &mut diagnostics);
    }

    diagnostics
}
