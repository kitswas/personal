use crate::models::TemplateMeta;
use serde::{Deserialize, Serialize};

/// Defines how to map a CSV/Excel row into a transaction leg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTemplate {
	pub meta: TemplateMeta,

	/// True if the first row contains headers and should be skipped
	pub skip_header: bool,

	/// 0-indexed column mappings
	pub mapping: ColumnMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
	/// Date column index
	pub date: usize,
	/// Date format string (e.g. "%Y-%m-%d" or "%d/%m/%Y")
	pub date_format: String,

	/// Payee column index
	pub payee: usize,

	/// Column index for the transaction amount.
	pub amount: usize,

	/// If true, amount column represents an outflow (positive means expense).
	/// If false, amount column represents an inflow (positive means income).
	pub invert_amount: bool,

	/// The default commodity to use for this template.
	pub commodity: String,
}
