use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    BuilderMissingField(&'static str),
    Http(reqwest::Error),
    Json(serde_json::Error),
    EmptyResponse,
    RequestFailed {
        status: reqwest::StatusCode,
        body: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuilderMissingField(field) => {
                write!(f, "missing required builder field: {field}")
            }
            Self::Http(error) => write!(f, "http error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
            Self::EmptyResponse => write!(f, "received empty response from server"),
            Self::RequestFailed { status, body } => {
                write!(f, "request failed with status {status}: {body}")
            }
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}
