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

/// Error returned by lifecycle-safe methods.
///
/// HTTP responses retain their status, raw body and Asaas rate-limit reset
/// header even when the body cannot be decoded as the expected DTO.
#[derive(Debug)]
pub enum LifecycleError {
    Http(reqwest::Error),
    Response {
        status: reqwest::StatusCode,
        body: String,
        retry_after: Option<String>,
        rate_limit_reset_seconds: Option<u64>,
        decode_error: Option<serde_json::Error>,
    },
}

impl LifecycleError {
    #[must_use]
    pub const fn status(&self) -> Option<reqwest::StatusCode> {
        match self {
            Self::Http(_) => None,
            Self::Response { status, .. } => Some(*status),
        }
    }

    #[must_use]
    pub fn response_body(&self) -> Option<&str> {
        match self {
            Self::Http(_) => None,
            Self::Response { body, .. } => Some(body),
        }
    }

    /// Returns the native `Retry-After` value without interpreting a duration or HTTP date.
    #[must_use]
    pub fn retry_after(&self) -> Option<&str> {
        match self {
            Self::Http(_) => None,
            Self::Response { retry_after, .. } => retry_after.as_deref(),
        }
    }

    #[must_use]
    pub const fn rate_limit_reset_seconds(&self) -> Option<u64> {
        match self {
            Self::Http(_) => None,
            Self::Response {
                rate_limit_reset_seconds,
                ..
            } => *rate_limit_reset_seconds,
        }
    }

    #[must_use]
    pub fn api_errors(&self) -> Option<Vec<ApiError>> {
        let Self::Response { body, .. } = self else {
            return None;
        };

        serde_json::from_str::<ApiErrorResponse>(body)
            .ok()
            .map(|response| response.errors)
    }
}

impl Display for LifecycleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(error) => write!(f, "http error: {error}"),
            Self::Response {
                status,
                body,
                decode_error: None,
                ..
            } => write!(f, "request failed with status {status}: {body}"),
            Self::Response {
                status,
                decode_error: Some(error),
                ..
            } => write!(f, "failed to decode response with status {status}: {error}"),
        }
    }
}

impl std::error::Error for LifecycleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(error) => Some(error),
            Self::Response {
                decode_error: Some(error),
                ..
            } => Some(error),
            Self::Response {
                decode_error: None, ..
            } => None,
        }
    }
}

impl From<reqwest::Error> for LifecycleError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
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
        retry_after: Option<String>,
        rate_limit_reset_seconds: Option<u64>,
    },
}

impl Error {
    #[must_use]
    pub const fn status(&self) -> Option<reqwest::StatusCode> {
        match self {
            Self::RequestFailed { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Returns the native `Retry-After` value without interpreting a duration or HTTP date.
    #[must_use]
    pub fn retry_after(&self) -> Option<&str> {
        match self {
            Self::RequestFailed { retry_after, .. } => retry_after.as_deref(),
            _ => None,
        }
    }

    #[must_use]
    pub const fn rate_limit_reset_seconds(&self) -> Option<u64> {
        match self {
            Self::RequestFailed {
                rate_limit_reset_seconds,
                ..
            } => *rate_limit_reset_seconds,
            _ => None,
        }
    }

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
            Self::RequestFailed { status, body, .. } => {
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
            retry_after: None,
            rate_limit_reset_seconds: None,
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
