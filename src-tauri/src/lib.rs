#![deny(unsafe_code)]
// Deny unsafe in all application code per AGENTS.md §6 and ADR 0001.

pub mod classifier;
pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod parser;
pub mod template;

use std::sync::{Arc, Mutex};

/// Shared application state injected into every Tauri command handler.
pub struct AppState {
	/// The encrypted SQLite connection.
	/// `None` until `unlock()` succeeds.
	/// Wrapped in `Arc<Mutex<_>>` so it can be cloned across `spawn_blocking` closures.
	pub db: Arc<Mutex<Option<rusqlite::Connection>>>,

	/// Filesystem path to the database file.
	pub db_path: std::path::PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.invoke_handler(tauri::generate_handler![
			commands::accounts::list_accounts,
			commands::accounts::create_account,
			commands::accounts::update_account,
			commands::accounts::delete_account,
			commands::accounts::get_default_commodity,
			commands::security::is_onboarding_done,
			commands::security::unlock,
			commands::security::setup_master_password,
			commands::security::change_master_password,
			commands::transactions::list_transactions,
			commands::transactions::commit_transaction,
			commands::transactions::delete_transaction,
			commands::transactions::get_running_balances,
			commands::import::list_templates,
			commands::import::parse_statement,
			commands::import::commit_import_batch
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
