use thiserror::Error;

/// Errors that can arise from SQLite storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("record not found: id={id}")]
    NotFound { id: String },

    #[error("duplicate key: id={id}")]
    DuplicateKey { id: String },

    #[error("constraint violation: {message}")]
    Constraint { message: String },

    #[error("database connection error")]
    Connection {
        #[source]
        source: sqlx::Error,
    },

    #[error("migration error")]
    Migration {
        #[source]
        source: sqlx::migrate::MigrateError,
    },

    #[error("serialization error")]
    Serialization {
        #[source]
        source: serde_json::Error,
    },
}

/// Convenience alias used throughout this crate.
pub type Result<T> = std::result::Result<T, StorageError>;

impl From<sqlx::Error> for StorageError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => {
                // Caller should use find_by_id and handle Option; this covers
                // cases where a mandatory fetch returns nothing.
                StorageError::NotFound {
                    id: String::from("<unknown>"),
                }
            }
            sqlx::Error::Database(db_err) => {
                // SQLite UNIQUE constraint violations have code "2067" or
                // message containing "UNIQUE constraint failed".
                let msg = db_err.message().to_string();
                if msg.contains("UNIQUE constraint failed") {
                    StorageError::DuplicateKey { id: msg }
                } else {
                    StorageError::Constraint { message: msg }
                }
            }
            _ => StorageError::Connection { source: err },
        }
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Serialization { source: err }
    }
}

impl From<sqlx::migrate::MigrateError> for StorageError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        StorageError::Migration { source: err }
    }
}
