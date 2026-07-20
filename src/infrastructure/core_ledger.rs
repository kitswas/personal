use crate::domain::{
	ledger::{Ledger, LedgerError},
	models::{Posting, Transaction},
	parser::ParsedRow,
	storage::Storage,
};
use uuid::Uuid;

pub struct CoreLedger;

impl Ledger for CoreLedger {
	fn validate_and_commit(
		&self,
		rows: &[ParsedRow],
		storage: &dyn Storage,
	) -> Result<(), LedgerError> {
		for row in rows {
			match row {
				ParsedRow::Valid {
					timestamp,
					payee,
					amount,
					commodity,
					suggested_account_id,
					external_id,
					..
				} => {
					let account_id = match suggested_account_id {
						Some(id) => id.clone(),
						None => {
							return Err(LedgerError::ValidationError(format!(
								"Cannot commit transaction for '{}': missing a suggested account",
								payee
							)));
						},
					};

					let txn_id = Uuid::new_v4().to_string();
					let txn = Transaction {
						id: txn_id.clone(),
						timestamp: timestamp.clone(),
						payee: payee.clone(),
						notes: None,
						external_id: external_id.clone(),
					};

					// Double entry requires 2 postings that sum to 0.
					// In a real import, one side is the bank account, the other is the
					// category. Here we simulate importing from a bank account
					// ("assets:bank"). A positive amount on the bank statement means
					// bank balance increases (Debit Asset).
					let bank_posting = Posting {
						id: Uuid::new_v4().to_string(),
						transaction_id: txn_id.clone(),
						account_id: "assets:bank".to_string(), /* In a real app this
						                                        * would be tied to the
						                                        * import source */
						amount: *amount,
						commodity: commodity.clone(),
					};

					let offset_posting = Posting {
						id: Uuid::new_v4().to_string(),
						transaction_id: txn_id.clone(),
						account_id, // The categorized account (e.g. expenses:groceries)
						amount: -*amount,
						commodity: commodity.clone(),
					};

					storage.save_transaction_with_postings(
						&txn,
						&[bank_posting, offset_posting],
					)?;
				},
				ParsedRow::Invalid { error_reason, .. } => {
					return Err(LedgerError::ValidationError(format!(
						"Cannot commit with invalid rows present: {}",
						error_reason
					)));
				},
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::domain::{models::Account, storage::StorageError};
	use std::cell::RefCell;

	struct MockStorage {
		pub transactions: RefCell<Vec<(Transaction, Vec<Posting>)>>,
	}

	impl MockStorage {
		fn new() -> Self {
			Self {
				transactions: RefCell::new(Vec::new()),
			}
		}
	}

	impl Storage for MockStorage {
		fn init_db(&self) -> Result<(), StorageError> {
			Ok(())
		}

		fn save_account(&self, _account: &Account) -> Result<(), StorageError> {
			Ok(())
		}

		fn save_transaction_with_postings(
			&self,
			txn: &Transaction,
			postings: &[Posting],
		) -> Result<(), StorageError> {
			self.transactions
				.borrow_mut()
				.push((txn.clone(), postings.to_vec()));
			Ok(())
		}

		fn get_running_balances(&self) -> Result<Vec<(Account, i64)>, StorageError> {
			Ok(vec![])
		}
	}

	#[test]
	fn test_ledger_commit_valid_rows() {
		let mock_storage = MockStorage::new();
		let ledger = CoreLedger;

		let rows = vec![ParsedRow::Valid {
			row_idx: 1,
			timestamp: "2024-01-01T12:00:00Z".to_string(),
			payee: "Grocery Store".to_string(),
			amount: -5000,
			commodity: "INR".to_string(),
			suggested_account_id: Some("expenses:food".to_string()),
			confidence: 1.0,
			external_id: None,
		}];

		assert!(ledger.validate_and_commit(&rows, &mock_storage).is_ok());

		let saved = mock_storage.transactions.borrow();
		assert_eq!(saved.len(), 1);

		let (txn, postings) = &saved[0];
		assert_eq!(txn.payee, "Grocery Store");
		assert_eq!(postings.len(), 2);

		let bank_p = postings
			.iter()
			.find(|p| p.account_id == "assets:bank")
			.unwrap();
		let expense_p = postings
			.iter()
			.find(|p| p.account_id == "expenses:food")
			.unwrap();

		assert_eq!(bank_p.amount, -5000);
		assert_eq!(expense_p.amount, 5000);
		assert_eq!(bank_p.amount + expense_p.amount, 0); // Double entry invariant holds
	}

	#[test]
	fn test_ledger_commit_invalid_rows_fails() {
		let mock_storage = MockStorage::new();
		let ledger = CoreLedger;

		let rows = vec![ParsedRow::Invalid {
			row_idx: 2,
			raw_data: "bad data".to_string(),
			error_reason: "Malformed format".to_string(),
		}];

		let result = ledger.validate_and_commit(&rows, &mock_storage);
		assert!(matches!(result, Err(LedgerError::ValidationError(_))));
	}
}
