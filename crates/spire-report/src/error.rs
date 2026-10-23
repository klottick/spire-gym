use std::{fmt, io};

#[derive(Debug)]
pub enum ReportError {
    Io(io::Error),
    Json(serde_json::Error),
    Invalid(String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f,"report I/O: {error}"),
            Self::Json(error) => write!(f,"report JSON: {error}"),
            Self::Invalid(message) => write!(f,"invalid report: {message}"),
        }
    }
}

impl std::error::Error for ReportError {}
impl From<io::Error> for ReportError { fn from(error:io::Error)->Self { Self::Io(error) } }
impl From<serde_json::Error> for ReportError { fn from(error:serde_json::Error)->Self { Self::Json(error) } }
