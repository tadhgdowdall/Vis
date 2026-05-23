use std::{fs, process};

use vis_analyzer::analyze;
use vis_diagnostics::render_diagnostic;
use vis_parser::{parse_html, parse_jsx};
use vis_rules::run_all;

use crate::error::CliError;

pub fn run(args: impl IntoIterator<Item = String>) -> Result<i32, CliError> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Err(CliError::Usage);
    };

    if command != "check" {
        return Err(CliError::Usage);
    }

    let Some(file_path) = args.next() else {
        return Err(CliError::Usage);
    };

    let source = fs::read_to_string(&file_path).map_err(|source| CliError::ReadFile {
        path: file_path.clone(),
        source,
    })?;

    let parsed = match file_path.rsplit('.').next() {
        Some("html") => parse_html(&source),
        Some("jsx") | Some("tsx") => parse_jsx(&source),
        _ => return Err(CliError::UnsupportedFileType(file_path)),
    }
    .map_err(CliError::Parse)?;

    let analyzed = analyze(&parsed);
    let diagnostics = run_all(&file_path, &analyzed);

    if diagnostics.is_empty() {
        println!("No accessibility issues found.");
        return Ok(0);
    }

    for diagnostic in diagnostics {
        println!("{}", render_diagnostic(&diagnostic, &source));
        println!();
    }

    Ok(1)
}

pub fn exit(result: Result<i32, CliError>) -> ! {
    match result {
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            process::exit(2);
        }
    }
}
