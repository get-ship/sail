//! Op errors — Connect-compatible codes + MCP/CLI messages.

use serde_json::{Value, json};
use thiserror::Error;

/// Handler-level failure.
#[derive(Debug, Error)]
pub enum OpError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("failed precondition: {0}")]
    FailedPrecondition(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl OpError {
    /// Connect error code (string enum used by Connect JSON unary).
    pub fn connect_code(&self) -> &'static str {
        match self {
            Self::InvalidArgument(_) => "invalid_argument",
            Self::NotFound(_) => "not_found",
            Self::PermissionDenied(_) => "permission_denied",
            Self::FailedPrecondition(_) => "failed_precondition",
            Self::Internal(_) => "internal",
        }
    }

    pub fn http_status(&self) -> u16 {
        match self {
            Self::InvalidArgument(_) => 400,
            Self::NotFound(_) => 404,
            Self::PermissionDenied(_) => 403,
            Self::FailedPrecondition(_) => 400,
            Self::Internal(_) => 500,
        }
    }

    /// Connect unary JSON error body.
    pub fn connect_body(&self) -> Value {
        json!({
            "code": self.connect_code(),
            "message": self.to_string(),
        })
    }
}

impl From<std::io::Error> for OpError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(e.to_string())
    }
}
