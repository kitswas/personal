use serde::Serialize;
use ts_rs::TS;

/// Typed error enum for all application errors.
///
/// Every variant maps to a human-readable string via [`std::fmt::Display`].
/// Tauri commands serialize errors as `String` at the IPC boundary — callers
/// receive the `Display` representation.
#[derive(Debug, thiserror::Error, Serialize, TS)]
#[serde(tag = "type", content = "message")]
#[ts(export, export_to = "../../src/types/ipc_bindings.ts")]
pub enum AppError {
	/// SQLite / rusqlite error.
	#[error("Database error: {0}")]
	Db(String),

	/// The master password supplied by the user is incorrect.
	#[error("Incorrect master password")]
	WrongPassword,

	/// A transaction's postings do not sum to zero.
	#[error("Transaction is unbalanced: sum of postings is {sum} (expected 0)")]
	UnbalancedTransaction { sum: i64 },

	/// An account referenced in a posting does not exist.
	#[error("Account not found: {id}")]
	AccountNotFound { id: String },

	/// A transaction referenced does not exist.
	#[error("Transaction not found: {id}")]
	TransactionNotFound { id: String },

	/// The import template file could not be parsed.
	#[error("Template parse error: {0}")]
	TemplateParse(String),

	/// The import file could not be read or parsed.
	#[error("Import parse error: {0}")]
	ImportParse(String),

	/// An error from the OS keyring (credential store).
	#[error("Keyring error: {0}")]
	Keyring(String),

	/// A generic I/O error (file operations).
	#[error("I/O error: {0}")]
	Io(String),

	/// Catch-all for errors that cannot be categorised more specifically.
	#[error("{0}")]
	Other(String),
}

impl From<AppError> for String {
	fn from(e: AppError) -> String {
		e.to_string()
	}
}

impl From<rusqlite::Error> for AppError {
	fn from(e: rusqlite::Error) -> Self {
		AppError::Db(e.to_string())
	}
}

impl From<std::io::Error> for AppError {
	fn from(e: std::io::Error) -> Self {
		AppError::Io(e.to_string())
	}
}

impl From<keyring::Error> for AppError {
	fn from(e: keyring::Error) -> Self {
		AppError::Keyring(e.to_string())
	}
}
