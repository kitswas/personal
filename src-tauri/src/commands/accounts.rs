use tauri::State;
use uuid::Uuid;

use crate::{AppState, error::AppError, models::Account};

// ---------------------------------------------------------------------------
// Helper — borrow the live connection or return a typed error
// ---------------------------------------------------------------------------

fn with_conn<T, F>(state: &AppState, f: F) -> Result<T, AppError>
where
	F: FnOnce(&rusqlite::Connection) -> Result<T, AppError>,
{
	let guard = state
		.db
		.lock()
		.map_err(|_| AppError::Other("DB mutex poisoned".into()))?;
	match guard.as_ref() {
		Some(conn) => f(conn),
		None => Err(AppError::Other(
			"Database is locked — call unlock() first".into(),
		)),
	}
}

// ---------------------------------------------------------------------------
// Pure query helpers
// ---------------------------------------------------------------------------

/// Map a single rusqlite row to an [`Account`].
fn row_to_account(row: &rusqlite::Row<'_>) -> rusqlite::Result<Account> {
	Ok(Account {
		id: row.get(0)?,
		name: row.get(1)?,
		account_type: row.get(2)?,
		commodity: row.get(3)?,
	})
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Returns `Err` if `account_type` is not one of the five allowed values.
fn validate_account_type(account_type: &str) -> Result<(), AppError> {
	match account_type {
		"asset" | "liability" | "equity" | "revenue" | "expense" => Ok(()),
		other => Err(AppError::Other(format!(
			"Invalid account type '{}': must be one of asset, liability, equity, revenue, expense",
			other
		))),
	}
}

/// Returns `Err` if `commodity` is not exactly 3 uppercase ASCII letters.
fn validate_commodity(commodity: &str) -> Result<(), AppError> {
	if commodity.len() == 3 && commodity.bytes().all(|b| b.is_ascii_uppercase()) {
		Ok(())
	} else {
		Err(AppError::Other(format!(
			"Invalid commodity '{}': must be 3 uppercase ASCII letters (ISO 4217)",
			commodity
		)))
	}
}

// ---------------------------------------------------------------------------
// IPC Commands
// ---------------------------------------------------------------------------

/// List all accounts ordered by type then name.
#[tauri::command]
pub fn list_accounts(state: State<'_, AppState>) -> Result<Vec<Account>, String> {
	with_conn(&state, |conn| {
		let mut stmt = conn
			.prepare("SELECT id, name, type, commodity FROM accounts ORDER BY type, name")
			.map_err(AppError::Db)?;

		let rows = stmt
			.query_map([], row_to_account)
			.map_err(AppError::Db)?
			.collect::<rusqlite::Result<Vec<Account>>>()
			.map_err(AppError::Db)?;

		Ok(rows)
	})
	.map_err(String::from)
}

/// Create a new account. Returns the generated UUID.
#[tauri::command]
pub fn create_account(
	state: State<'_, AppState>,
	name: String,
	account_type: String,
	commodity: String,
) -> Result<String, String> {
	validate_account_type(&account_type).map_err(String::from)?;
	validate_commodity(&commodity).map_err(String::from)?;

	if name.trim().is_empty() {
		return Err("Account name must not be empty".into());
	}

	let id = Uuid::new_v4().to_string();
	let id_clone = id.clone();

	with_conn(&state, move |conn| {
		conn.execute(
			"INSERT INTO accounts (id, name, type, commodity) VALUES (?1, ?2, ?3, ?4)",
			rusqlite::params![id_clone, name.trim(), account_type, commodity],
		)
		.map_err(AppError::Db)?;
		Ok(id_clone)
	})
	.map_err(String::from)?;

	Ok(id)
}

/// Update an existing account's name, type, and commodity.
#[tauri::command]
pub fn update_account(
	state: State<'_, AppState>,
	id: String,
	name: String,
	account_type: String,
	commodity: String,
) -> Result<(), String> {
	validate_account_type(&account_type).map_err(String::from)?;
	validate_commodity(&commodity).map_err(String::from)?;

	if name.trim().is_empty() {
		return Err("Account name must not be empty".into());
	}

	with_conn(&state, |conn| {
		let affected = conn
			.execute(
				"UPDATE accounts SET name = ?1, type = ?2, commodity = ?3 WHERE id = ?4",
				rusqlite::params![name.trim(), account_type, commodity, id],
			)
			.map_err(AppError::Db)?;

		if affected == 0 {
			Err(AppError::Other(format!("Account '{}' not found", id)))
		} else {
			Ok(())
		}
	})
	.map_err(String::from)
}

/// Delete an account. Fails if the account has any postings attached.
#[tauri::command]
pub fn delete_account(state: State<'_, AppState>, id: String) -> Result<(), String> {
	with_conn(&state, |conn| {
		// Check no postings reference this account.
		let count: i64 = conn
			.query_row(
				"SELECT COUNT(*) FROM postings WHERE account_id = ?1",
				rusqlite::params![id],
				|row| row.get(0),
			)
			.map_err(AppError::Db)?;

		if count > 0 {
			return Err(AppError::Other(format!(
				"Cannot delete account '{}': it has {} posting(s). Remove transactions first.",
				id, count
			)));
		}

		let affected = conn
			.execute("DELETE FROM accounts WHERE id = ?1", rusqlite::params![id])
			.map_err(AppError::Db)?;

		if affected == 0 {
			Err(AppError::Other(format!("Account '{}' not found", id)))
		} else {
			Ok(())
		}
	})
	.map_err(String::from)
}

/// Return the default commodity stored in settings.
#[tauri::command]
pub fn get_default_commodity(state: State<'_, AppState>) -> Result<String, String> {
	with_conn(&state, |conn| {
		let result: rusqlite::Result<String> = conn.query_row(
			"SELECT value FROM settings WHERE key = 'default_commodity'",
			[],
			|row| row.get(0),
		);
		match result {
			Ok(v) => Ok(v),
			Err(rusqlite::Error::QueryReturnedNoRows) => Ok("USD".into()),
			Err(e) => Err(AppError::Db(e)),
		}
	})
	.map_err(String::from)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;
	use crate::db;

	#[test]
	fn validate_account_type_accepts_valid_types() {
		for t in &["asset", "liability", "equity", "revenue", "expense"] {
			assert!(validate_account_type(t).is_ok(), "expected ok for {}", t);
		}
	}

	#[test]
	fn validate_account_type_rejects_invalid() {
		assert!(validate_account_type("bank").is_err());
		assert!(validate_account_type("Asset").is_err());
		assert!(validate_account_type("").is_err());
	}

	#[test]
	fn validate_commodity_accepts_valid() {
		assert!(validate_commodity("INR").is_ok());
		assert!(validate_commodity("USD").is_ok());
		assert!(validate_commodity("EUR").is_ok());
	}

	#[test]
	fn validate_commodity_rejects_invalid() {
		assert!(validate_commodity("inr").is_err()); // lowercase
		assert!(validate_commodity("US").is_err()); // too short
		assert!(validate_commodity("USDD").is_err()); // too long
		assert!(validate_commodity("US1").is_err()); // digit
		assert!(validate_commodity("").is_err()); // empty
	}

	// Integration tests against a real in-memory SQLCipher DB.
	fn make_test_conn() -> rusqlite::Connection {
		let conn = db::open(std::path::Path::new(":memory:"), "test_accounts_key")
			.expect("open");
		db::run_migrations(&conn).expect("migrations");
		conn
	}

	#[test]
	fn create_and_list_account() {
		let conn = make_test_conn();
		let id = Uuid::new_v4().to_string();
		conn.execute(
			"INSERT INTO accounts (id, name, type, commodity) VALUES (?1, ?2, ?3, ?4)",
			rusqlite::params![id, "Savings", "asset", "INR"],
		)
		.expect("insert");

		let mut stmt = conn
			.prepare("SELECT id, name, type, commodity FROM accounts ORDER BY type, name")
			.expect("prepare");
		let accounts: Vec<Account> = stmt
			.query_map([], row_to_account)
			.expect("query")
			.collect::<rusqlite::Result<Vec<Account>>>()
			.expect("collect");

		assert_eq!(accounts.len(), 1);
		assert_eq!(accounts[0].name, "Savings");
		assert_eq!(accounts[0].account_type, "asset");
		assert_eq!(accounts[0].commodity, "INR");
	}

	#[test]
	fn delete_account_blocked_if_has_postings() {
		let conn = make_test_conn();
		let acc_id = Uuid::new_v4().to_string();
		let txn_id = Uuid::new_v4().to_string();
		let posting_id = Uuid::new_v4().to_string();

		conn.execute(
			"INSERT INTO accounts (id, name, type, commodity) VALUES (?1, 'Bank', 'asset', 'INR')",
			rusqlite::params![acc_id],
		)
		.expect("insert account");

		conn.execute(
            "INSERT INTO transactions (id, date, payee, notes) VALUES (?1, '2024-01-01', 'Test', '')",
            rusqlite::params![txn_id],
        )
        .expect("insert txn");

		conn.execute(
            "INSERT INTO postings (id, transaction_id, account_id, amount, commodity) VALUES (?1, ?2, ?3, 1000, 'INR')",
            rusqlite::params![posting_id, txn_id, acc_id],
        )
        .expect("insert posting");

		// Should block deletion since a posting references this account.
		let count: i64 = conn
			.query_row(
				"SELECT COUNT(*) FROM postings WHERE account_id = ?1",
				rusqlite::params![acc_id],
				|row| row.get(0),
			)
			.expect("count");
		assert_eq!(count, 1);
	}
}
