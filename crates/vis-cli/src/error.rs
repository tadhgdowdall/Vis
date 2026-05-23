use std::{error::Error, fmt, io};

use vis_parser::ParseError;

#[derive(Debug)]
pub enum CliError {
    Usage,
    ReadFile { path: String, source: io::Error },
    UnsupportedFileType(String),
    Parse(ParseError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => f.write_str("usage: vis check <file.html|file.jsx|file.tsx>"),
            Self::ReadFile { path, source } => {
                write!(f, "failed to read {path}: {source}")
            }
            Self::UnsupportedFileType(path) => {
                write!(f, "unsupported file type for {path}")
            }
            Self::Parse(error) => write!(f, "parse error: {error}"),
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadFile { source, .. } => Some(source),
            Self::Parse(error) => Some(error),
            Self::Usage | Self::UnsupportedFileType(_) => None,
        }
    }
}
