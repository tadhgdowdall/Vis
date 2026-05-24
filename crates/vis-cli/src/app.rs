use std::collections::{BTreeMap, HashMap};
use std::{fs, path::Path, process};

use vis_analyzer::analyze_with_map;
use vis_diagnostics::{Diagnostic, render_diagnostic, render_diagnostics_json};
use vis_parser::{parse_html, parse_jsx};
use vis_rules::run_all;
use walkdir::WalkDir;

use crate::config::Config;
use crate::error::CliError;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    ".next",
    "target",
    "__pycache__",
    ".venv",
    "venv",
    "coverage",
    ".idea",
    ".vscode",
];

const SUPPORTED_EXTS: &[&str] = &["html", "jsx", "tsx", "js"];

struct FileReport {
    _path: String,
    source: String,
    diagnostics: Vec<Diagnostic>,
}

pub fn run(args: impl IntoIterator<Item = String>) -> Result<i32, CliError> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Err(CliError::Usage);
    };

    if command != "check" {
        return Err(CliError::Usage);
    }

    let mut json = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        if arg == "--json" {
            json = true;
        } else {
            paths.push(arg);
        }
    }

    let paths = if paths.is_empty() {
        vec![".".to_string()]
    } else {
        paths
    };

    let config = Config::load().unwrap_or_default();

    check_targets(&paths, &config.components, json)
}

fn check_targets(
    paths: &[String],
    component_map: &HashMap<String, String>,
    json: bool,
) -> Result<i32, CliError> {
    let mut files_checked = 0usize;
    let mut reports: Vec<FileReport> = Vec::new();
    let mut error_count = 0usize;

    for path_str in paths {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("error: {}: no such file or directory", path_str);
            error_count += 1;
            continue;
        }

        if path.is_dir() {
            scan_directory(
                path,
                component_map,
                &mut files_checked,
                &mut reports,
                &mut error_count,
            );
        } else if path.is_file() {
            match check_file(path, component_map) {
                Ok((source, diagnostics)) => {
                    files_checked += 1;
                    if !diagnostics.is_empty() {
                        reports.push(FileReport {
                            _path: path.to_string_lossy().into(),
                            source,
                            diagnostics,
                        });
                    }
                }
                Err(e) => {
                    eprintln!("error: {}: {}", path.display(), e);
                    error_count += 1;
                }
            }
        }
    }

    if files_checked == 0 {
        return Err(CliError::NoTargets);
    }

    let total_issues: usize = reports.iter().map(|r| r.diagnostics.len()).sum();

    if json {
        let source_diag_pairs: Vec<(&str, &[Diagnostic])> = reports
            .iter()
            .map(|r| (r.source.as_str(), r.diagnostics.as_slice()))
            .collect();
        let json_output = render_diagnostics_json(&source_diag_pairs);
        println!("{json_output}");
    } else {
        if total_issues > 0 {
            print_summary(&reports);
            println!();
        }

        for report in &reports {
            for diagnostic in &report.diagnostics {
                println!("{}", render_diagnostic(diagnostic, &report.source));
                println!();
            }
        }
    }

    println!();
    if total_issues > 0 {
        if json {
            eprintln!(
                "{} file{} scanned, {} issue{} found",
                files_checked,
                if files_checked == 1 { "" } else { "s" },
                total_issues,
                if total_issues == 1 { "" } else { "s" },
            );
        } else {
            println!(
                "{} file{} scanned, {} issue{} found",
                files_checked,
                if files_checked == 1 { "" } else { "s" },
                total_issues,
                if total_issues == 1 { "" } else { "s" },
            );
        }
        Ok(1)
    } else {
        println!(
            "{} file{} scanned, no issues found",
            files_checked,
            if files_checked == 1 { "" } else { "s" },
        );
        Ok(0)
    }
}

fn print_summary(reports: &[FileReport]) {
    let mut by_code: BTreeMap<&str, usize> = BTreeMap::new();

    for report in reports {
        for diagnostic in &report.diagnostics {
            *by_code.entry(&diagnostic.code).or_default() += 1;
        }
    }

    let mut entries: Vec<(&str, usize)> = by_code.into_iter().collect();
    entries.sort_by_key(|(code, count)| (std::cmp::Reverse(*count), *code));

    let max_width = entries
        .iter()
        .map(|(code, _)| code.len())
        .max()
        .unwrap_or(0);

    for (code, count) in &entries {
        let label = if *count == 1 { "issue " } else { "issues" };
        println!(
            "  {:<width$} {:>4} {}",
            code,
            count,
            label,
            width = max_width
        );
    }
}

fn check_file(
    path: &Path,
    component_map: &HashMap<String, String>,
) -> Result<(String, Vec<Diagnostic>), String> {
    let source = fs::read_to_string(path).map_err(|e| format!("{e}"))?;

    let parsed = match path.extension().and_then(|e| e.to_str()) {
        Some("html") => parse_html(&source),
        Some("jsx") | Some("tsx") | Some("js") => parse_jsx(&source),
        _ => return Err("unsupported file type".to_string()),
    }
    .map_err(|e| format!("{e}"))?;

    let map = if component_map.is_empty() {
        None
    } else {
        Some(component_map.clone())
    };

    let analyzed = analyze_with_map(&parsed, map);
    let diagnostics = run_all(&path.to_string_lossy(), &analyzed);

    Ok((source, diagnostics))
}

fn scan_directory(
    dir: &Path,
    component_map: &HashMap<String, String>,
    files_checked: &mut usize,
    reports: &mut Vec<FileReport>,
    error_count: &mut usize,
) {
    let walker = WalkDir::new(dir).into_iter().filter_entry(|e| {
        !e.file_type().is_dir()
            || e.file_name()
                .to_str()
                .is_none_or(|name| !SKIP_DIRS.contains(&name) && !name.starts_with('.'))
    });

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("error: {}", e);
                *error_count += 1;
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !SUPPORTED_EXTS.contains(&ext) {
            continue;
        }

        match check_file(path, component_map) {
            Ok((source, diagnostics)) => {
                *files_checked += 1;
                if !diagnostics.is_empty() {
                    reports.push(FileReport {
                        _path: path.to_string_lossy().into(),
                        source,
                        diagnostics,
                    });
                }
            }
            Err(e) => {
                eprintln!("error: {}: {}", path.display(), e);
                *error_count += 1;
            }
        }
    }
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
