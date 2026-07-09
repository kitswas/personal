use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

use crate::error::AppError;

// ---------------------------------------------------------------------------
// Key constants
// ---------------------------------------------------------------------------

/// Keyring service name — used to store and retrieve the master DB key.
pub const KEYRING_SERVICE: &str = "io.github.kitswas.personal";
/// Keyring username — the account under which the key is stored.
pub const KEYRING_USER: &str = "master_key";

// ---------------------------------------------------------------------------
// Connection setup
// ---------------------------------------------------------------------------

/// Open the encrypted SQLite database at `db_path`.
///
/// Steps performed:
/// 1. Open (or create) the file.
/// 2. Apply the encryption key via `PRAGMA key`.
/// 3. Run a test query to validate the key (wrong key → `AppError::WrongPassword`).
/// 4. Enable WAL mode for crash-safe writes.
/// 5. Run all migrations idempotently.
///
/// # Errors
/// Returns [`AppError::WrongPassword`] if the key is invalid.
/// Returns [`AppError::Db`] for any other SQLite error.
pub fn open(db_path: &Path, key: &str) -> Result<Connection, AppError> {
	let conn = Connection::open(db_path).map_err(AppError::Db)?;

	// Apply encryption key — must be the very first pragma sent.
	let key_pragma = format!("PRAGMA key = '{}';", key.replace('\'', "''"));
	conn.execute_batch(&key_pragma).map_err(AppError::Db)?;

	// Validate the key by running a trivial query.
	// If the key is wrong, SQLCipher returns SQLITE_NOTADB here.
	conn.execute_batch("SELECT count(*) FROM sqlite_master;")
		.map_err(|e| match e {
			rusqlite::Error::SqliteFailure(ref fe, _)
				if fe.extended_code == rusqlite::ffi::SQLITE_NOTADB =>
			{
				AppError::WrongPassword
			},
			other => AppError::Db(other),
		})?;

	// WAL mode: writes do not block readers; crash-safe.
	conn.execute_batch("PRAGMA journal_mode = WAL;")
		.map_err(AppError::Db)?;

	// Foreign key enforcement.
	conn.execute_batch("PRAGMA foreign_keys = ON;")
		.map_err(AppError::Db)?;

	Ok(conn)
}

// ---------------------------------------------------------------------------
// Migrations
// ---------------------------------------------------------------------------

/// Run all migrations against an already-opened, already-keyed connection.
///
/// Migrations are idempotent (`CREATE TABLE IF NOT EXISTS`).
/// This function is safe to call on every application start.
pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
	conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
		.map_err(AppError::Db)
}

// ---------------------------------------------------------------------------
// Onboarding guard
// ---------------------------------------------------------------------------

/// Returns `true` only when `onboarding_complete = 'true'` is present in
/// the `settings` table.
///
/// Returns `false` if:
/// - The key is absent (fresh install or mid-onboarding crash).
/// - The value is anything other than the literal string `"true"`.
///
/// See ADR 0002 for the design rationale.
pub fn is_onboarding_done(conn: &Connection) -> Result<bool, AppError> {
	let result: SqlResult<String> = conn.query_row(
		"SELECT value FROM settings WHERE key = 'onboarding_complete'",
		[],
		|row| row.get(0),
	);
	match result {
		Ok(v) => Ok(v == "true"),
		Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
		Err(e) => Err(AppError::Db(e)),
	}
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;
	use tempfile::NamedTempFile;

	fn open_test_db() -> (Connection, NamedTempFile) {
		let tmp = NamedTempFile::new().expect("tempfile");
		let conn = open(tmp.path(), "test_key_1234").expect("open");
		run_migrations(&conn).expect("migrations");
		(conn, tmp)
	}

	#[test]
	fn migration_is_idempotent() {
		let (conn, _tmp) = open_test_db();
		// Running migrations a second time must not error.
		run_migrations(&conn).expect("second migration run");
	}

	#[test]
	fn wal_mode_is_enabled() {
		let (conn, _tmp) = open_test_db();
		let mode: String = conn
			.query_row("PRAGMA journal_mode", [], |r| r.get(0))
			.expect("journal_mode");
		assert_eq!(mode, "wal");
	}

	#[test]
	fn onboarding_done_absent_key_returns_false() {
		let (conn, _tmp) = open_test_db();
		assert!(!is_onboarding_done(&conn).expect("is_onboarding_done"));
	}

	#[test]
	fn onboarding_done_true_returns_true() {
		let (conn, _tmp) = open_test_db();
		conn.execute(
			"INSERT INTO settings (key, value) VALUES ('onboarding_complete', 'true')",
			[],
		)
		.expect("insert");
		assert!(is_onboarding_done(&conn).expect("is_onboarding_done"));
	}

	#[test]
	fn onboarding_done_wrong_value_returns_false() {
		let (conn, _tmp) = open_test_db();
		conn.execute(
			"INSERT INTO settings (key, value) VALUES ('onboarding_complete', 'false')",
			[],
		)
		.expect("insert");
		assert!(!is_onboarding_done(&conn).expect("is_onboarding_done"));
	}

	#[test]
	fn wrong_password_returns_error() {
		let tmp = NamedTempFile::new().expect("tempfile");
		// Create a real encrypted DB with one key.
		{
			let conn = open(tmp.path(), "correct_key").expect("open with correct key");
			run_migrations(&conn).expect("migrations");
		}
		// Re-open with the wrong key — must return WrongPassword, not panic.
		let result = open(tmp.path(), "wrong_key");
		assert!(
			matches!(result, Err(AppError::WrongPassword)),
			"expected WrongPassword, got {:?}",
			result
		);
	}
}
