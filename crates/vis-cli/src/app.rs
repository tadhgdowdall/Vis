use std::collections::{BTreeMap, HashMap};
use std::{env, fs, path::Path, process};

use vis_analyzer::analyze_with_map;
use vis_diagnostics::{Diagnostic, Severity, render_diagnostic, render_diagnostics_json};
use vis_parser::{parse_html, parse_jsx};
use vis_rules::run_all_with_config;
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

    let config = Config::load()?;

    check_targets(
        &paths,
        &config.components,
        &config.rules,
        &config.exclude,
        json,
    )
}

fn check_targets(
    paths: &[String],
    component_map: &HashMap<String, String>,
    rule_config: &HashMap<String, Severity>,
    exclude_patterns: &[String],
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

        if is_excluded(path, exclude_patterns) {
            continue;
        }

        if path.is_dir() {
            scan_directory(
                path,
                component_map,
                rule_config,
                exclude_patterns,
                &mut files_checked,
                &mut reports,
                &mut error_count,
            );
        } else if path.is_file() {
            match check_file(path, component_map, rule_config) {
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
                    files_checked += 1;
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
    let error_issues: usize = reports
        .iter()
        .flat_map(|r| &r.diagnostics)
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();

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

    let has_issues = total_issues > 0 || error_count > 0;
    let has_failing_issues = error_issues > 0 || error_count > 0;

    println!();
    if has_issues {
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
        Ok(if has_failing_issues { 1 } else { 0 })
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
    rule_config: &HashMap<String, Severity>,
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
    let diagnostics = run_all_with_config(&path.to_string_lossy(), &analyzed, rule_config);

    Ok((source, diagnostics))
}

fn scan_directory(
    dir: &Path,
    component_map: &HashMap<String, String>,
    rule_config: &HashMap<String, Severity>,
    exclude_patterns: &[String],
    files_checked: &mut usize,
    reports: &mut Vec<FileReport>,
    error_count: &mut usize,
) {
    let walker = WalkDir::new(dir).into_iter().filter_entry(|e| {
        if is_excluded_from(e.path(), dir, exclude_patterns) {
            return false;
        }

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
        if is_excluded_from(path, dir, exclude_patterns) {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !SUPPORTED_EXTS.contains(&ext) {
            continue;
        }

        match check_file(path, component_map, rule_config) {
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
                *files_checked += 1;
                eprintln!("error: {}: {}", path.display(), e);
                *error_count += 1;
            }
        }
    }
}

fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    is_excluded_by_path(normalize_path(path), patterns)
}

fn is_excluded_from(path: &Path, base: &Path, patterns: &[String]) -> bool {
    let normalized = path
        .strip_prefix(base)
        .ok()
        .map(normalize_path)
        .unwrap_or_else(|| normalize_path(path));
    is_excluded_by_path(normalized, patterns)
}

fn is_excluded_by_path(normalized: String, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    patterns.iter().any(|pattern| {
        let pattern = normalize_pattern(pattern);
        path_matches_pattern(&normalized, &pattern)
    })
}

fn normalize_path(path: &Path) -> String {
    let path = if path.is_absolute() {
        env::current_dir()
            .ok()
            .and_then(|cwd| path.strip_prefix(cwd).ok().map(Path::to_path_buf))
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    };

    path.to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn normalize_pattern(pattern: &str) -> String {
    pattern
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }

    let directory_pattern = pattern.strip_suffix("/**").unwrap_or(pattern);
    if path == directory_pattern || path.starts_with(&format!("{directory_pattern}/")) {
        return true;
    }

    wildcard_match(pattern.as_bytes(), path.as_bytes())
}

fn wildcard_match(pattern: &[u8], value: &[u8]) -> bool {
    let (mut pattern_index, mut value_index) = (0usize, 0usize);
    let mut star_index = None;
    let mut retry_value_index = 0usize;

    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?' || pattern[pattern_index] == value[value_index])
        {
            pattern_index += 1;
            value_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star_index = Some(pattern_index);
            pattern_index += 1;
            retry_value_index = value_index;
        } else if let Some(star) = star_index {
            pattern_index = star + 1;
            retry_value_index += 1;
            value_index = retry_value_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
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

#[cfg(test)]
mod tests {
    use super::check_targets;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("vis-cli-{name}-{id}"));
        fs::create_dir_all(&dir).expect("temp directory should be created");
        dir
    }

    #[test]
    fn warning_only_diagnostics_do_not_fail_check() {
        let dir = temp_dir("warnings");
        let file = dir.join("warning.html");
        fs::write(&file, "<h1></h1>").expect("fixture should be written");

        let code = check_targets(
            &[file.to_string_lossy().into_owned()],
            &HashMap::new(),
            &HashMap::new(),
            &[],
            false,
        )
        .expect("check should complete");

        assert_eq!(code, 0);
        fs::remove_dir_all(dir).expect("temp directory should be removed");
    }

    #[test]
    fn error_diagnostics_fail_check() {
        let dir = temp_dir("errors");
        let file = dir.join("error.html");
        fs::write(&file, r#"<img src="/hero.png" />"#).expect("fixture should be written");

        let code = check_targets(
            &[file.to_string_lossy().into_owned()],
            &HashMap::new(),
            &HashMap::new(),
            &[],
            false,
        )
        .expect("check should complete");

        assert_eq!(code, 1);
        fs::remove_dir_all(dir).expect("temp directory should be removed");
    }

    #[test]
    fn excludes_matching_directory_paths() {
        let dir = temp_dir("exclude");
        let skip_dir = dir.join("skip");
        fs::create_dir_all(&skip_dir).expect("skip directory should be created");
        fs::write(skip_dir.join("error.html"), r#"<img src="/hero.png" />"#)
            .expect("excluded fixture should be written");
        fs::write(
            dir.join("pass.html"),
            r#"<img src="/hero.png" alt="Hero" />"#,
        )
        .expect("included fixture should be written");

        let code = check_targets(
            &[dir.to_string_lossy().into_owned()],
            &HashMap::new(),
            &HashMap::new(),
            &["skip/**".to_string()],
            false,
        )
        .expect("check should complete");

        assert_eq!(code, 0);
        fs::remove_dir_all(dir).expect("temp directory should be removed");
    }
}
