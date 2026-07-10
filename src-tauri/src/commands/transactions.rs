use tauri::State;
use uuid::Uuid;

use crate::{
	AppState,
	error::AppError,
	ipc::IpcResponse,
	models::{BalanceEntry, Posting, PostingInput, Transaction, TransactionWithPostings},
};

// ---------------------------------------------------------------------------
// Helper
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
// IPC Commands
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Inner logic
// ---------------------------------------------------------------------------

fn list_transactions_inner(
	state: &AppState,
	limit: u32,
	offset: u32,
) -> Result<Vec<TransactionWithPostings>, String> {
	with_conn(state, |conn| {
        let mut stmt = conn
            .prepare("SELECT id, date, payee, notes FROM transactions ORDER BY date DESC LIMIT ?1 OFFSET ?2")
            .map_err(AppError::from)?;
        
        let txns: Vec<Transaction> = stmt
            .query_map(rusqlite::params![limit, offset], |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    payee: row.get(2)?,
                    notes: row.get(3)?,
                })
            })
            .map_err(AppError::from)?
            .collect::<rusqlite::Result<Vec<Transaction>>>()
            .map_err(AppError::from)?;

        let mut result = Vec::with_capacity(txns.len());
        let mut post_stmt = conn
            .prepare("SELECT id, transaction_id, account_id, amount, commodity FROM postings WHERE transaction_id = ?1")
            .map_err(AppError::from)?;

        for txn in txns {
            let postings = post_stmt
                .query_map(rusqlite::params![txn.id], |row| {
                    Ok(Posting {
                        id: row.get(0)?,
                        transaction_id: row.get(1)?,
                        account_id: row.get(2)?,
                        amount: row.get(3)?,
                        commodity: row.get(4)?,
                    })
                })
                .map_err(AppError::from)?
                .collect::<rusqlite::Result<Vec<Posting>>>()
                .map_err(AppError::from)?;

            result.push(TransactionWithPostings {
                transaction: txn,
                postings,
            });
        }

        Ok(result)
    })
    .map_err(String::from)
}

fn commit_transaction_inner(
	state: &AppState,
	date: String,
	payee: String,
	notes: String,
	postings: Vec<PostingInput>,
) -> Result<Transaction, String> {
	if postings.is_empty() {
		return Err("Transaction must have at least one posting".into());
	}

	let sum: i64 = postings.iter().map(|p| p.amount).sum();
	if sum != 0 {
		return Err(format!(
			"Transaction postings must sum to 0. Current sum is {}",
			sum
		));
	}

	let txn_id = Uuid::new_v4().to_string();
	let txn_id_clone = txn_id.clone();

	let txn = Transaction {
		id: txn_id.clone(),
		date: date.clone(),
		payee: payee.clone(),
		notes: notes.clone(),
	};

	with_conn(state, move |conn| {
        conn.execute("BEGIN", []).map_err(AppError::from)?;

        if let Err(e) = (|| -> Result<(), rusqlite::Error> {
            conn.execute(
                "INSERT INTO transactions (id, date, payee, notes) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![txn_id_clone, date, payee, notes],
            )?;

            for posting in postings {
                let posting_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO postings (id, transaction_id, account_id, amount, commodity) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        posting_id,
                        txn_id_clone,
                        posting.account_id,
                        posting.amount,
                        posting.commodity,
                    ],
                )?;
            }
            Ok(())
        })() {
            let _ = conn.execute("ROLLBACK", []);
            return Err(AppError::from(e));
        }

        conn.execute("COMMIT", []).map_err(AppError::from)?;
        Ok(txn)
    })
    .map_err(String::from)
}

fn delete_transaction_inner(state: &AppState, id: String) -> Result<(), String> {
	with_conn(state, |conn| {
		let affected = conn
			.execute(
				"DELETE FROM transactions WHERE id = ?1",
				rusqlite::params![id],
			)
			.map_err(AppError::from)?;

		if affected == 0 {
			Err(AppError::Other(format!("Transaction '{}' not found", id)))
		} else {
			Ok(())
		}
	})
	.map_err(String::from)
}

fn get_running_balances_inner(
	state: &AppState,
	account_id: String,
) -> Result<Vec<BalanceEntry>, String> {
	with_conn(state, |conn| {
		let mut stmt = conn
			.prepare(
				r#"
                SELECT t.date, SUM(p.amount)
                FROM postings p
                JOIN transactions t ON p.transaction_id = t.id
                WHERE p.account_id = ?1
                GROUP BY t.date
                ORDER BY t.date ASC
                "#,
			)
			.map_err(AppError::from)?;

		let daily_deltas = stmt
			.query_map(rusqlite::params![account_id], |row| {
				let date: String = row.get(0)?;
				let delta: i64 = row.get(1)?;
				Ok((date, delta))
			})
			.map_err(AppError::from)?
			.collect::<rusqlite::Result<Vec<(String, i64)>>>()
			.map_err(AppError::from)?;

		let mut balances = Vec::with_capacity(daily_deltas.len());
		let mut running = 0i64;

		for (date, delta) in daily_deltas {
			running += delta;
			balances.push(BalanceEntry {
				date,
				balance: running,
			});
		}

		Ok(balances)
	})
	.map_err(String::from)
}

// ---------------------------------------------------------------------------
// IPC Commands
// ---------------------------------------------------------------------------

/// List transactions with their postings, ordered by date descending.
#[tauri::command]
pub fn list_transactions(
	state: State<'_, AppState>,
	limit: u32,
	offset: u32,
) -> IpcResponse<Vec<TransactionWithPostings>, AppError> {
	list_transactions_inner(&state, limit, offset)
		.map_err(|e| AppError::Other(e))
		.into()
}

/// Commit a new double-entry transaction.
#[tauri::command]
pub fn commit_transaction(
	state: State<'_, AppState>,
	date: String,
	payee: String,
	notes: String,
	postings: Vec<PostingInput>,
) -> IpcResponse<Transaction, AppError> {
	commit_transaction_inner(&state, date, payee, notes, postings)
		.map_err(|e| AppError::Other(e))
		.into()
}

/// Delete a transaction by ID.
#[tauri::command]
pub fn delete_transaction(
	state: State<'_, AppState>,
	id: String,
) -> IpcResponse<(), AppError> {
	delete_transaction_inner(&state, id)
		.map_err(|e| AppError::Other(e))
		.into()
}

/// Get running balances for an account, grouped by date.
#[tauri::command]
pub fn get_running_balances(
	state: State<'_, AppState>,
	account_id: String,
) -> IpcResponse<Vec<BalanceEntry>, AppError> {
	get_running_balances_inner(&state, account_id)
		.map_err(|e| AppError::Other(e))
		.into()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;
	use crate::db;
	use std::sync::{Arc, Mutex};
	use tempfile::NamedTempFile;

	fn test_state() -> (AppState, NamedTempFile) {
		let tmp = NamedTempFile::new().expect("tempfile");
		let conn = db::open(tmp.path(), "test_key").expect("open");
		db::run_migrations(&conn).expect("migrations");

		let state = AppState {
			db_path: tmp.path().to_path_buf(),
			db: Arc::new(Mutex::new(Some(conn))),
		};
		(state, tmp)
	}

	#[test]
	fn commit_transaction_fails_if_sum_not_zero() {
		let (state, _tmp) = test_state();
		let postings = vec![
			PostingInput {
				account_id: "a".into(),
				amount: 100,
				commodity: "INR".into(),
			},
			PostingInput {
				account_id: "b".into(),
				amount: -50,
				commodity: "INR".into(),
			},
		];

		let res = commit_transaction_inner(
			&state,
			"2024-01-01".into(),
			"Payee".into(),
			"".into(),
			postings,
		);
		assert!(res.is_err());
		assert_eq!(
			res.unwrap_err(),
			"Transaction postings must sum to 0. Current sum is 50"
		);
	}

	#[test]
	fn commit_transaction_success() {
		let (state, _tmp) = test_state();

		// Need to create dummy accounts first because of foreign key constraints
		{
			let conn_guard = state.db.lock().unwrap();
			let conn = conn_guard.as_ref().unwrap();
			conn.execute("INSERT INTO accounts (id, name, type, commodity) VALUES ('a', 'Acc A', 'asset', 'INR')", []).unwrap();
			conn.execute("INSERT INTO accounts (id, name, type, commodity) VALUES ('b', 'Acc B', 'expense', 'INR')", []).unwrap();
		}

		let postings = vec![
			PostingInput {
				account_id: "a".into(),
				amount: -100,
				commodity: "INR".into(),
			},
			PostingInput {
				account_id: "b".into(),
				amount: 100,
				commodity: "INR".into(),
			},
		];

		let txn = commit_transaction_inner(
			&state,
			"2024-01-01".into(),
			"Payee".into(),
			"".into(),
			postings,
		)
		.unwrap();

		let list = list_transactions_inner(&state, 10, 0).unwrap();
		assert_eq!(list.len(), 1);
		assert_eq!(list[0].transaction.id, txn.id);
		assert_eq!(list[0].postings.len(), 2);
	}
}
