use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Core domain models — serialised across the IPC boundary
// ---------------------------------------------------------------------------

/// A ledger account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
	pub id: String,
	pub name: String,
	/// One of: "asset" | "liability" | "equity" | "revenue" | "expense"
	pub account_type: String,
	/// ISO 4217 currency code, e.g. "INR"
	pub commodity: String,
}

/// A single posting (one leg of a double-entry transaction).
/// `amount` is in the smallest currency unit (paise for INR, cents for USD).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Posting {
	pub id: String,
	pub transaction_id: String,
	pub account_id: String,
	/// Signed integer — smallest currency unit.
	pub amount: i64,
	/// ISO 4217 currency code.
	pub commodity: String,
}

/// A transaction header (without postings).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
	pub id: String,
	/// ISO 8601 date string: "YYYY-MM-DD"
	pub date: String,
	pub payee: String,
	pub notes: String,
}

/// A transaction together with its postings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionWithPostings {
	pub transaction: Transaction,
	pub postings: Vec<Posting>,
}

/// Input type for creating a new posting (no id yet — generated server-side).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostingInput {
	pub account_id: String,
	pub amount: i64,
	pub commodity: String,
}

/// A single entry in an account's running balance history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceEntry {
	/// ISO 8601 date string.
	pub date: String,
	/// Running balance after this date's transactions, in smallest unit.
	pub balance: i64,
}

// ---------------------------------------------------------------------------
// Import pipeline models
// ---------------------------------------------------------------------------

/// The result of parsing a single row from an imported statement.
///
/// All matches on this enum must be exhaustive — the compiler enforces this.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ParsedRow {
	/// Row was parsed successfully and passes validation.
	Valid {
		row_idx: usize,
		/// ISO 8601 date string.
		date: String,
		payee: String,
		/// Amount in smallest currency unit.
		amount: i64,
		commodity: String,
		/// Suggested account from auto-categorisation; may be empty.
		suggested_account_id: String,
		/// Confidence score 0.0–1.0 from the Naive Bayes classifier.
		confidence: f32,
	},
	/// Row could not be parsed or failed validation.
	Invalid {
		row_idx: usize,
		/// Raw cell data joined for display.
		raw_data: String,
		/// Human-readable reason for rejection.
		error_reason: String,
	},
}

/// A row that has been reviewed and confirmed as valid, ready for commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidRow {
	pub date: String,
	pub payee: String,
	pub amount: i64,
	pub commodity: String,
	pub account_id: String,
	/// The offset account (e.g. the bank asset account being imported into).
	pub offset_account_id: String,
}

/// Result of committing an import batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
	pub committed: usize,
	pub failed: usize,
}

// ---------------------------------------------------------------------------
// Template metadata
// ---------------------------------------------------------------------------

/// Minimal metadata about a bundled or user-defined import template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateMeta {
	pub name: String,
	pub description: String,
	pub institution: String,
}
