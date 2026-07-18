use crate::domain::{
	models::{Account, AccountType, Posting, Transaction},
	storage::{Storage, StorageError},
};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::PathBuf;

pub struct SqliteStorage {
	db_path: PathBuf,
	encryption_key: String,
}

impl SqliteStorage {
	pub fn new(db_path: PathBuf, encryption_key: String) -> Self {
		Self {
			db_path,
			encryption_key,
		}
	}

	fn get_connection(&self) -> Result<Connection, StorageError> {
		let conn = Connection::open(&self.db_path)
			.map_err(|e| StorageError::DbError(format!("Failed to open DB: {}", e)))?;

		conn.pragma_update(None, "key", &self.encryption_key)
			.map_err(|e| {
				StorageError::DbError(format!("Failed to set encryption key: {}", e))
			})?;

		// Verify key is correct
		conn.execute_batch("SELECT count(*) FROM sqlite_master;")
			.map_err(|e| {
				StorageError::DbError(format!(
					"Invalid encryption key or corrupted DB: {}",
					e
				))
			})?;

		Ok(conn)
	}
}

impl Storage for SqliteStorage {
	fn init_db(&self) -> Result<(), StorageError> {
		let conn = self.get_connection()?;

		conn.execute_batch(
			r#"
			CREATE TABLE IF NOT EXISTS settings (
				key   TEXT PRIMARY KEY,
				value TEXT NOT NULL
			);

			CREATE TABLE IF NOT EXISTS accounts (
				id        TEXT PRIMARY KEY,
				name      TEXT NOT NULL,
				type      TEXT NOT NULL CHECK(type IN ('asset','liability','equity','revenue','expense')),
				commodity TEXT NOT NULL DEFAULT 'INR'
			);

			CREATE TABLE IF NOT EXISTS transactions (
				id    TEXT PRIMARY KEY,
				date  TEXT NOT NULL,
				payee TEXT NOT NULL,
				notes TEXT
			);

			CREATE TABLE IF NOT EXISTS postings (
				id             TEXT PRIMARY KEY,
				transaction_id TEXT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
				account_id     TEXT NOT NULL REFERENCES accounts(id),
				amount         INTEGER NOT NULL,
				commodity      TEXT NOT NULL DEFAULT 'INR'
			);

			CREATE INDEX IF NOT EXISTS idx_postings_txn     ON postings(transaction_id);
			CREATE INDEX IF NOT EXISTS idx_postings_account ON postings(account_id);
			CREATE INDEX IF NOT EXISTS idx_txn_date         ON transactions(date);
			"#,
		).map_err(|e| StorageError::DbError(format!("Failed to initialize schema: {}", e)))?;

		Ok(())
	}

	fn save_account(&self, account: &Account) -> Result<(), StorageError> {
		let conn = self.get_connection()?;
		let type_str = match account.account_type {
			AccountType::Asset => "asset",
			AccountType::Liability => "liability",
			AccountType::Equity => "equity",
			AccountType::Revenue => "revenue",
			AccountType::Expense => "expense",
		};

		conn.execute(
			"INSERT INTO accounts (id, name, type, commodity) VALUES (?1, ?2, ?3, ?4)
			 ON CONFLICT(id) DO UPDATE SET name=excluded.name, type=excluded.type, commodity=excluded.commodity",
			params![account.id, account.name, type_str, account.commodity],
		).map_err(|e| StorageError::DbError(format!("Failed to save account: {}", e)))?;

		Ok(())
	}

	fn save_transaction_with_postings(
		&self,
		txn: &Transaction,
		postings: &[Posting],
	) -> Result<(), StorageError> {
		let mut conn = self.get_connection()?;
		let tx = conn
			.transaction()
			.map_err(|e| StorageError::DbError(e.to_string()))?;

		// 1. Verify balancing rule
		let sum: i64 = postings.iter().map(|p| p.amount).sum();
		if sum != 0 {
			return Err(StorageError::IntegrityError(format!(
				"Transaction {} does not balance. Sum: {}",
				txn.id, sum
			)));
		}

		// 2. Insert transaction
		tx.execute(
			"INSERT INTO transactions (id, date, payee, notes) VALUES (?1, ?2, ?3, ?4)
			 ON CONFLICT(id) DO UPDATE SET date=excluded.date, payee=excluded.payee, notes=excluded.notes",
			params![txn.id, txn.date, txn.payee, txn.notes],
		).map_err(|e| StorageError::DbError(format!("Failed to save transaction: {}", e)))?;

		// 3. Clear existing postings for this transaction to allow clean updates
		tx.execute(
			"DELETE FROM postings WHERE transaction_id = ?1",
			params![txn.id],
		)
		.map_err(|e| {
			StorageError::DbError(format!("Failed to clear old postings: {}", e))
		})?;

		// 4. Insert postings
		for posting in postings {
			tx.execute(
				"INSERT INTO postings (id, transaction_id, account_id, amount, commodity) VALUES (?1, ?2, ?3, ?4, ?5)",
				params![posting.id, posting.transaction_id, posting.account_id, posting.amount, posting.commodity],
			).map_err(|e| StorageError::DbError(format!("Failed to save posting {}: {}", posting.id, e)))?;
		}

		tx.commit().map_err(|e| {
			StorageError::DbError(format!("Failed to commit transaction: {}", e))
		})?;

		Ok(())
	}

	fn get_running_balances(&self) -> Result<Vec<(String, i64)>, StorageError> {
		let conn = self.get_connection()?;
		let mut stmt = conn
			.prepare(
				"SELECT account_id, SUM(amount) as balance FROM postings GROUP BY account_id",
			)
			.map_err(|e| StorageError::DbError(e.to_string()))?;

		let balances_iter = stmt
			.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
			.map_err(|e| StorageError::DbError(e.to_string()))?;

		let mut results = Vec::new();
		for b in balances_iter {
			results.push(b.map_err(|e| StorageError::DbError(e.to_string()))?);
		}

		Ok(results)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::NamedTempFile;
	use uuid::Uuid;

	fn get_test_storage() -> SqliteStorage {
		let temp_file = NamedTempFile::new().unwrap();
		let db_path = temp_file.into_temp_path().to_path_buf();
		let key = Uuid::new_v4().to_string();
		let storage = SqliteStorage::new(db_path, key);
		storage.init_db().unwrap();
		storage
	}

	#[test]
	fn test_init_db() {
		let storage = get_test_storage();
		let conn = storage.get_connection().unwrap();
		let count: i64 = conn
			.query_row(
				"SELECT count(*) FROM sqlite_master WHERE type='table'",
				[],
				|r| r.get(0),
			)
			.unwrap();
		assert!(count >= 4); // settings, accounts, transactions, postings
	}

	#[test]
	fn test_save_account() {
		let storage = get_test_storage();
		let acc = Account {
			id: "acc_1".to_string(),
			name: "Checking".to_string(),
			account_type: AccountType::Asset,
			commodity: "INR".to_string(),
		};

		assert!(storage.save_account(&acc).is_ok());

		let conn = storage.get_connection().unwrap();
		let name: String = conn
			.query_row("SELECT name FROM accounts WHERE id = 'acc_1'", [], |r| {
				r.get(0)
			})
			.unwrap();
		assert_eq!(name, "Checking");
	}

	#[test]
	fn test_save_transaction_valid() {
		let storage = get_test_storage();
		let acc1 = Account {
			id: "acc_1".to_string(),
			name: "Checking".to_string(),
			account_type: AccountType::Asset,
			commodity: "INR".to_string(),
		};
		let acc2 = Account {
			id: "acc_2".to_string(),
			name: "Groceries".to_string(),
			account_type: AccountType::Expense,
			commodity: "INR".to_string(),
		};
		storage.save_account(&acc1).unwrap();
		storage.save_account(&acc2).unwrap();

		let txn = Transaction {
			id: "txn_1".to_string(),
			date: "2024-01-01".to_string(),
			payee: "Supermarket".to_string(),
			notes: None,
		};

		let p1 = Posting {
			id: "post_1".to_string(),
			transaction_id: "txn_1".to_string(),
			account_id: "acc_1".to_string(),
			amount: -5000,
			commodity: "INR".to_string(),
		};

		let p2 = Posting {
			id: "post_2".to_string(),
			transaction_id: "txn_1".to_string(),
			account_id: "acc_2".to_string(),
			amount: 5000,
			commodity: "INR".to_string(),
		};

		assert!(
			storage
				.save_transaction_with_postings(&txn, &[p1, p2])
				.is_ok()
		);

		let balances = storage.get_running_balances().unwrap();
		assert_eq!(balances.len(), 2);
		let checking_bal = balances.iter().find(|(id, _)| id == "acc_1").unwrap().1;
		let grocery_bal = balances.iter().find(|(id, _)| id == "acc_2").unwrap().1;

		assert_eq!(checking_bal, -5000);
		assert_eq!(grocery_bal, 5000);
	}

	#[test]
	fn test_save_transaction_unbalanced() {
		let storage = get_test_storage();
		let acc1 = Account {
			id: "acc_1".to_string(),
			name: "Checking".to_string(),
			account_type: AccountType::Asset,
			commodity: "INR".to_string(),
		};
		storage.save_account(&acc1).unwrap();

		let txn = Transaction {
			id: "txn_1".to_string(),
			date: "2024-01-01".to_string(),
			payee: "Supermarket".to_string(),
			notes: None,
		};

		let p1 = Posting {
			id: "post_1".to_string(),
			transaction_id: "txn_1".to_string(),
			account_id: "acc_1".to_string(),
			amount: -5000,
			commodity: "INR".to_string(),
		};

		// Unbalanced (Sum != 0)
		let result = storage.save_transaction_with_postings(&txn, &[p1]);
		assert!(matches!(result, Err(StorageError::IntegrityError(_))));
	}
}
