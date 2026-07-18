use super::{
	parser::ParsedRow,
	storage::{Storage, StorageError},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
	#[error("Validation error: {0}")]
	ValidationError(String),
	#[error("Storage error: {0}")]
	StorageError(#[from] StorageError),
}

/// The Core Ledger Contract defining business logic boundaries
pub trait Ledger {
	/// Validates parsed rows (e.g. enforcing SUM(amount) = 0) and commits them via
	/// Storage
	fn validate_and_commit(
		&self,
		rows: &[ParsedRow],
		storage: &dyn Storage,
	) -> Result<(), LedgerError>;
}
