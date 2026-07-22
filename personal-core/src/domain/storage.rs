use super::models::{Account, Posting, Transaction};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
	#[error("Database error: {0}")]
	DbError(String),
	#[error("Not found: {0}")]
	NotFound(String),
	#[error("Integrity error: {0}")]
	IntegrityError(String),
}

/// The Storage Contract defining boundaries for data persistence
pub trait Storage {
	fn init_db(&self) -> Result<(), StorageError>;

	fn save_account(&self, account: &Account) -> Result<(), StorageError>;

	fn save_transaction_with_postings(
		&self,
		txn: &Transaction,
		postings: &[Posting],
	) -> Result<(), StorageError>;

	fn get_running_balances(&self) -> Result<Vec<(Account, i64)>, StorageError>;
}
