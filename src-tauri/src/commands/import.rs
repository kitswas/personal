use std::path::Path;
use tauri::State;
use uuid::Uuid;

use crate::{
	AppState,
	error::AppError,
	models::{BatchResult, ParsedRow, TemplateMeta, ValidRow},
	template::types::{ColumnMapping, ImportTemplate},
};

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

/// Dummy data for templates for now. In reality, these might be stored in the DB or config files.
fn get_builtin_templates() -> Vec<ImportTemplate> {
	vec![
		ImportTemplate {
			meta: TemplateMeta {
				name: "HDFC Bank".into(),
				description: "Standard HDFC Bank statement".into(),
				institution: "HDFC".into(),
			},
			skip_header: true,
			mapping: ColumnMapping {
				date: 0,
				date_format: "%d/%m/%y".into(),
				payee: 1,
				amount: 4, // Assume credit/debit combined or just one column
				invert_amount: false,
				commodity: "INR".into(),
			},
		},
		ImportTemplate {
			meta: TemplateMeta {
				name: "SBI".into(),
				description: "State Bank of India".into(),
				institution: "SBI".into(),
			},
			skip_header: true,
			mapping: ColumnMapping {
				date: 0,
				date_format: "%d %b %Y".into(),
				payee: 2,
				amount: 5,
				invert_amount: false,
				commodity: "INR".into(),
			},
		},
	]
}

#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> Result<Vec<TemplateMeta>, String> {
	let _ = state;
	Ok(get_builtin_templates()
		.into_iter()
		.map(|t| t.meta)
		.collect())
}

#[tauri::command]
pub fn parse_statement(
	state: State<'_, AppState>,
	file_path: String,
	template_name: String,
) -> Result<Vec<ParsedRow>, String> {
	let _ = state; // We don't strictly need DB here yet, unless we run the classifier.

	let template = get_builtin_templates()
		.into_iter()
		.find(|t| t.meta.name == template_name)
		.ok_or_else(|| format!("Template '{}' not found", template_name))?;

	let path = Path::new(&file_path);
	crate::parser::parse_statement(path, &template).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn commit_import_batch(
	state: State<'_, AppState>,
	rows: Vec<ValidRow>,
) -> Result<BatchResult, String> {
	if rows.is_empty() {
		return Ok(BatchResult {
			committed: 0,
			failed: 0,
		});
	}

	with_conn(&state, move |conn| {
        conn.execute("BEGIN", []).map_err(AppError::Db)?;

        let mut committed = 0;
        let mut failed = 0;

        for row in rows {
            let res = (|| -> Result<(), rusqlite::Error> {
                let txn_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO transactions (id, date, payee, notes) VALUES (?1, ?2, ?3, '')",
                    rusqlite::params![txn_id, row.date, row.payee],
                )?;

                // Posting 1: Category / Expense Account
                let posting_1 = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO postings (id, transaction_id, account_id, amount, commodity) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        posting_1,
                        txn_id,
                        row.account_id,
                        row.amount,
                        row.commodity,
                    ],
                )?;

                // Posting 2: Offset / Bank Account
                let posting_2 = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO postings (id, transaction_id, account_id, amount, commodity) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        posting_2,
                        txn_id,
                        row.offset_account_id,
                        -row.amount,
                        row.commodity,
                    ],
                )?;

                Ok(())
            })();

            if res.is_ok() {
                committed += 1;
            } else {
                failed += 1;
            }
        }

        conn.execute("COMMIT", []).map_err(AppError::Db)?;
        Ok(BatchResult { committed, failed })
    })
    .map_err(String::from)
}
