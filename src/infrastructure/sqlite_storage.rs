use crate::domain::{
	models::{Account, AccountType, Posting, Transaction},
	storage::{Storage, StorageError},
};
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct SqliteStorage {
	db_path: PathBuf,
	encryption_key: String,
}

impl std::fmt::Debug for SqliteStorage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("SqliteStorage")
			.field("db_path", &self.db_path)
			.field("encryption_key", &"***REDACTED***")
			.finish()
	}
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

	pub fn is_onboarding_done(&self) -> Result<bool, StorageError> {
		let conn = self.get_connection()?;
		let result: rusqlite::Result<String> = conn.query_row(
			"SELECT value FROM settings WHERE key = 'onboarding_complete'",
			[],
			|row| row.get(0),
		);
		match result {
			Ok(v) => Ok(v == "true"),
			Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
			Err(e) => Err(StorageError::DbError(e.to_string())),
		}
	}

	pub fn complete_onboarding(&self, base_commodity: &str) -> Result<(), StorageError> {
		let mut conn = self.get_connection()?;
		let tx = conn
			.transaction()
			.map_err(|e| StorageError::DbError(e.to_string()))?;

		tx.execute(
			"INSERT OR REPLACE INTO settings (key, value) VALUES ('onboarding_complete', 'true')",
			[],
		)
		.map_err(|e| StorageError::DbError(e.to_string()))?;

		tx.execute(
			"INSERT OR REPLACE INTO settings (key, value) VALUES ('base_commodity', ?1)",
			params![base_commodity],
		)
		.map_err(|e| StorageError::DbError(e.to_string()))?;

		tx.commit()
			.map_err(|e| StorageError::DbError(e.to_string()))?;
		Ok(())
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
				commodity TEXT NOT NULL DEFAULT 'INR',
				is_active BOOLEAN NOT NULL DEFAULT 1
			);

			CREATE TABLE IF NOT EXISTS transactions (
				id          TEXT PRIMARY KEY,
				timestamp   TEXT NOT NULL,
				payee       TEXT NOT NULL,
				notes       TEXT,
				external_id TEXT UNIQUE
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
			CREATE INDEX IF NOT EXISTS idx_txn_timestamp    ON transactions(timestamp);
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
			"INSERT INTO accounts (id, name, type, commodity, is_active) VALUES (?1, ?2, ?3, ?4, ?5)
			 ON CONFLICT(id) DO UPDATE SET name=excluded.name, type=excluded.type, commodity=excluded.commodity, is_active=excluded.is_active",
			params![account.id, account.name, type_str, account.commodity, account.is_active],
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

		// 1. Verify balancing rule PER COMMODITY
		let mut balances_by_commodity = std::collections::HashMap::new();
		for posting in postings {
			*balances_by_commodity.entry(&posting.commodity).or_insert(0) +=
				posting.amount;
		}
		for (commodity, sum) in balances_by_commodity {
			if sum != 0 {
				return Err(StorageError::IntegrityError(format!(
					"Transaction {} does not balance for commodity {}. Sum: {}",
					txn.id, commodity, sum
				)));
			}
		}

		// 2. Insert transaction
		tx.execute(
			"INSERT INTO transactions (id, timestamp, payee, notes, external_id) VALUES (?1, ?2, ?3, ?4, ?5)
			 ON CONFLICT(id) DO UPDATE SET timestamp=excluded.timestamp, payee=excluded.payee, notes=excluded.notes, external_id=excluded.external_id",
			params![txn.id, txn.timestamp, txn.payee, txn.notes, txn.external_id],
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

	fn get_running_balances(&self) -> Result<Vec<(Account, i64)>, StorageError> {
		let conn = self.get_connection()?;
		let mut stmt = conn
			.prepare(
				"SELECT a.id, a.name, a.type, a.commodity, a.is_active, SUM(p.amount) as balance
                 FROM accounts a 
                 JOIN postings p ON a.id = p.account_id 
                 GROUP BY a.id"
			)
			.map_err(|e| StorageError::DbError(e.to_string()))?;

		let balances_iter = stmt
			.query_map([], |row| {
				let type_str: String = row.get(2)?;
				let account_type = match type_str.as_str() {
					"asset" => AccountType::Asset,
					"liability" => AccountType::Liability,
					"equity" => AccountType::Equity,
					"revenue" => AccountType::Revenue,
					"expense" => AccountType::Expense,
					other => panic!("Invalid account type in DB: {}", other),
				};
				let acc = Account {
					id: row.get(0)?,
					name: row.get(1)?,
					account_type,
					commodity: row.get(3)?,
					is_active: row.get(4)?,
				};
				Ok((acc, row.get(5)?))
			})
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
			is_active: true,
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
			is_active: true,
		};
		let acc2 = Account {
			id: "acc_2".to_string(),
			name: "Groceries".to_string(),
			account_type: AccountType::Expense,
			commodity: "INR".to_string(),
			is_active: true,
		};
		storage.save_account(&acc1).unwrap();
		storage.save_account(&acc2).unwrap();

		let txn = Transaction {
			id: "txn_1".to_string(),
			timestamp: "2024-01-01T12:00:00Z".to_string(),
			payee: "Supermarket".to_string(),
			notes: None,
			external_id: None,
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
		let checking_bal = balances.iter().find(|(a, _)| a.id == "acc_1").unwrap().1;
		let grocery_bal = balances.iter().find(|(a, _)| a.id == "acc_2").unwrap().1;

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
			is_active: true,
		};
		storage.save_account(&acc1).unwrap();

		let txn = Transaction {
			id: "txn_1".to_string(),
			timestamp: "2024-01-01T12:00:00Z".to_string(),
			payee: "Supermarket".to_string(),
			notes: None,
			external_id: None,
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

	#[test]
	fn test_save_transaction_duplicate_external_id() {
		let storage = get_test_storage();
		let acc1 = Account {
			id: "acc_1".to_string(),
			name: "Checking".to_string(),
			account_type: AccountType::Asset,
			commodity: "INR".to_string(),
			is_active: true,
		};
		storage.save_account(&acc1).unwrap();

		let txn1 = Transaction {
			id: "txn_1".to_string(),
			timestamp: "2024-01-01T12:00:00Z".to_string(),
			payee: "A".to_string(),
			notes: None,
			external_id: Some("ext_123".to_string()),
		};
		let p1 = Posting {
			id: "post_1".to_string(),
			transaction_id: "txn_1".to_string(),
			account_id: "acc_1".to_string(),
			amount: 0,
			commodity: "INR".to_string(),
		};
		storage
			.save_transaction_with_postings(&txn1, &[p1])
			.unwrap();

		let txn2 = Transaction {
			id: "txn_2".to_string(),
			timestamp: "2024-01-02T12:00:00Z".to_string(),
			payee: "B".to_string(),
			notes: None,
			external_id: Some("ext_123".to_string()),
		};
		let p2 = Posting {
			id: "post_2".to_string(),
			transaction_id: "txn_2".to_string(),
			account_id: "acc_1".to_string(),
			amount: 0,
			commodity: "INR".to_string(),
		};

		// Should fail due to UNIQUE constraint on external_id
		let result = storage.save_transaction_with_postings(&txn2, &[p2]);
		assert!(matches!(result, Err(StorageError::DbError(_))));
	}
}
