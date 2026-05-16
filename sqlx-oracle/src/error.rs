use std::fmt;

use sqlx_core::error::{DatabaseError, ErrorKind};

#[derive(Debug)]
pub struct OracleDbError {
    pub message: String,
}

impl OracleDbError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for OracleDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for OracleDbError {}

impl DatabaseError for OracleDbError {
    fn message(&self) -> &str {
        &self.message
    }

    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }

    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
}
