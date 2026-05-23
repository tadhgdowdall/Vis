use std::{env, fs, process};

use vis_analyzer::analyze;
use vis_parser::{parse_html, parse_jsx};
use vis_rules::run_all;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(usage());
    };

    if command != "check" {
        return Err(usage());
    }

    let Some(file_path) = args.next() else {
        return Err(usage());
    };

    let source = fs::read_to_string(&file_path)
        .map_err(|error| format!("failed to read {file_path}: {error}"))?;

    let parsed = match file_path.rsplit('.').next() {
        Some("html") => parse_html(&source),
        Some("jsx") | Some("tsx") => parse_jsx(&source),
        _ => Err(format!("unsupported file type for {file_path}")),
    }?;

    let analyzed = analyze(&parsed);
    let diagnostics = run_all(&file_path, &analyzed);

    if diagnostics.is_empty() {
        println!("No accessibility issues found.");
        return Ok(());
    }

    for diagnostic in diagnostics {
        println!("{}", diagnostic.render(&source));
        println!();
    }

    process::exit(1);
}

fn usage() -> String {
    "usage: vis check <file.html|file.jsx|file.tsx>".to_string()
}
