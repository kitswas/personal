use keyring::Entry;
use tauri::State;

use crate::{AppState, db, error::AppError};

// ---------------------------------------------------------------------------
// Keyring helpers (pure functions over the keyring service)
// ---------------------------------------------------------------------------

fn keyring_entry() -> Result<Entry, AppError> {
	Entry::new(db::KEYRING_SERVICE, db::KEYRING_USER).map_err(AppError::from)
}

fn store_key(key: &str) -> Result<(), AppError> {
	keyring_entry()?.set_password(key).map_err(AppError::from)
}

fn load_key() -> Result<String, AppError> {
	keyring_entry()?.get_password().map_err(AppError::from)
}

fn delete_key() -> Result<(), AppError> {
	keyring_entry()?.delete_credential().map_err(AppError::from)
}

// ---------------------------------------------------------------------------
// Helper — borrow the live connection or return a typed error
// ---------------------------------------------------------------------------

/// Lock the connection Mutex and call `f` with a reference to the open
/// [`rusqlite::Connection`].
///
/// Returns `Err(AppError::Other("Database is locked"))` if the DB has not
/// been unlocked yet.
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
// IPC Commands
// ---------------------------------------------------------------------------

/// Check whether the onboarding wizard has been completed.
///
/// Returns `false` if the key is absent or the DB has not been unlocked yet
/// (treated as "not done" so the UI routes to onboarding).
#[tauri::command]
pub fn is_onboarding_done(state: State<'_, AppState>) -> Result<bool, String> {
	with_conn(&state, db::is_onboarding_done).map_err(String::from)
}

/// Attempt to unlock the database with the master password stored in the OS keyring.
///
/// On first call after launch, opens the encrypted SQLite file and stores the
/// live connection in [`AppState`].
///
/// Returns `true` on success, `false` if the stored key does not match.
#[tauri::command]
pub fn unlock(state: State<'_, AppState>) -> Result<bool, String> {
	// If already unlocked, return early.
	{
		let guard = state
			.db
			.lock()
			.map_err(|_| "DB mutex poisoned".to_string())?;
		if guard.is_some() {
			return Ok(true);
		}
	}

	let key = match load_key() {
		Ok(k) => k,
		// No key in keyring yet — onboarding has not been completed.
		Err(_) => return Ok(false),
	};

	match db::open(&state.db_path, &key) {
		Ok(conn) => {
			db::run_migrations(&conn).map_err(String::from)?;
			let mut guard = state
				.db
				.lock()
				.map_err(|_| "DB mutex poisoned".to_string())?;
			*guard = Some(conn);
			Ok(true)
		},
		Err(AppError::WrongPassword) => Ok(false),
		Err(e) => Err(e.to_string()),
	}
}

/// Complete onboarding: store the master password in the OS keyring and
/// write the `onboarding_complete` flag atomically with the default commodity.
///
/// This is the ONLY place that writes `onboarding_complete = 'true'`.
/// If this command fails or is interrupted, onboarding will restart on next
/// launch (see ADR 0002).
#[tauri::command]
pub fn setup_master_password(
	state: State<'_, AppState>,
	password: String,
	default_currency: String,
) -> Result<(), String> {
	// Validate currency code: must be 3 uppercase ASCII letters.
	if !is_valid_currency_code(&default_currency) {
		return Err(format!(
			"Invalid currency code '{}': must be 3 uppercase ASCII letters",
			default_currency
		));
	}

	// Open (or create) the DB with the new password.
	let conn = db::open(&state.db_path, &password).map_err(String::from)?;
	db::run_migrations(&conn).map_err(String::from)?;

	// Write settings atomically: onboarding_complete + default_commodity.
	conn.execute_batch(&format!(
        "BEGIN;
         INSERT OR REPLACE INTO settings (key, value) VALUES ('default_commodity', '{currency}');
         INSERT OR REPLACE INTO settings (key, value) VALUES ('onboarding_complete', 'true');
         COMMIT;",
        currency = default_currency.replace('\'', "''")
    ))
    .map_err(|e| AppError::Db(e).to_string())?;

	// Store the key in the OS keyring.
	store_key(&password).map_err(String::from)?;

	// Install the connection into AppState.
	let mut guard = state
		.db
		.lock()
		.map_err(|_| "DB mutex poisoned".to_string())?;
	*guard = Some(conn);

	Ok(())
}

/// Change the master password.
///
/// Validates the old password against the keyring, then re-encrypts the
/// database with the new password via `PRAGMA rekey` and updates the keyring.
#[tauri::command]
pub fn change_master_password(
	state: State<'_, AppState>,
	old_password: String,
	new_password: String,
) -> Result<(), String> {
	// Verify the old key matches what we have stored.
	let stored_key = load_key().map_err(String::from)?;
	if stored_key != old_password {
		return Err(AppError::WrongPassword.to_string());
	}

	with_conn(&state, |conn| {
		let rekey_pragma =
			format!("PRAGMA rekey = '{}';", new_password.replace('\'', "''"));
		conn.execute_batch(&rekey_pragma).map_err(AppError::Db)
	})
	.map_err(String::from)?;

	// Update the keyring to the new key.
	delete_key().map_err(String::from)?;
	store_key(&new_password).map_err(String::from)?;

	Ok(())
}

// ---------------------------------------------------------------------------
// Pure validation helpers
// ---------------------------------------------------------------------------

/// Returns `true` if `code` is exactly 3 uppercase ASCII letters (ISO 4217).
fn is_valid_currency_code(code: &str) -> bool {
	code.len() == 3 && code.bytes().all(|b| b.is_ascii_uppercase())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn currency_code_validation() {
		assert!(is_valid_currency_code("INR"));
		assert!(is_valid_currency_code("USD"));
		assert!(is_valid_currency_code("EUR"));
		assert!(!is_valid_currency_code("inr")); // lowercase
		assert!(!is_valid_currency_code("US")); // too short
		assert!(!is_valid_currency_code("USDD")); // too long
		assert!(!is_valid_currency_code("US1")); // digit
		assert!(!is_valid_currency_code("")); // empty
	}
}
