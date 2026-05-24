use std::{error::Error, fmt};

#[derive(Debug)]
pub enum CliError {
    Usage,
    NoTargets,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => f.write_str(
                "usage: vis check [path ...]\n\n  path    file or directory to check (defaults to current directory)"
            ),
            Self::NoTargets => f.write_str(
                "no supported files found (looked for .html, .jsx, .tsx, .js)"
            ),
        }
    }
}

impl Error for CliError {}
