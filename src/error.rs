use std::fmt::{Display, Formatter};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ApiError {
    pub code: String,
    pub description: String,
}

#[derive(Deserialize)]
struct ApiErrorResponse {
    errors: Vec<ApiError>,
}

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

impl Error {
    pub fn api_errors(&self) -> Option<Vec<ApiError>> {
        let Self::RequestFailed { body, .. } = self else {
            return None;
        };

        serde_json::from_str::<ApiErrorResponse>(body)
            .ok()
            .map(|response| response.errors)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_structured_errors_from_failed_requests() {
        let error = Error::RequestFailed {
            status: reqwest::StatusCode::BAD_REQUEST,
            body: r#"{"errors":[{"code":"invalid_action","description":"Esta cobrança não pode mais ser paga."}]}"#
                .to_string(),
        };

        let errors = error.api_errors().expect("structured Asaas errors");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "invalid_action");
        assert_eq!(
            errors[0].description,
            "Esta cobrança não pode mais ser paga."
        );
    }
}
