/// Errors that can occur when interacting with the MuninnDB API.
#[derive(Debug, thiserror::Error)]
pub enum MuninnError {
    #[error("request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("server error {status}: {message}")]
    ServerError { status: u16, message: String },

    #[error("timeout")]
    Timeout,

    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("max retries exceeded: {0}")]
    MaxRetriesExceeded(String),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),
}

impl MuninnError {
    pub(crate) fn from_status(status: u16, body: &str) -> Self {
        let msg = extract_error_message(body);
        match status {
            401 => MuninnError::Unauthorized(msg),
            404 => MuninnError::NotFound(msg),
            400..=499 => MuninnError::Validation(msg),
            _ => MuninnError::ServerError {
                status,
                message: msg,
            },
        }
    }

    pub(crate) fn is_retryable(&self) -> bool {
        match self {
            MuninnError::ServerError { status, .. } => *status >= 500,
            MuninnError::Timeout | MuninnError::ConnectionFailed(_) => true,
            _ => false,
        }
    }
}

fn extract_error_message(body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(obj) = v.as_object() {
            if let Some(err) = obj.get("error") {
                if let Some(s) = err.as_str() {
                    return s.to_string();
                }
            }
            if let Some(msg) = obj.get("message") {
                if let Some(s) = msg.as_str() {
                    return s.to_string();
                }
            }
        }
    }
    if body.is_empty() {
        "empty response".to_string()
    } else {
        body.to_string()
    }
}
