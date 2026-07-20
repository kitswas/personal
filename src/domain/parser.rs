use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum ParsedRow {
	Valid {
		row_idx: usize,
		timestamp: String,
		payee: String,
		amount: i64,
		commodity: String,
		suggested_account_id: Option<String>,
		confidence: f32,
		external_id: Option<String>,
	},
	Invalid {
		row_idx: usize,
		raw_data: String,
		error_reason: String,
	},
}

#[derive(Debug, Clone)]
pub struct ImportTemplate {
	pub name: String,
	pub format: String,
	pub has_headers: bool,
	pub date_col: usize,
	pub date_format: String,
	pub payee_col: usize,
	pub amount_col: usize,
	pub amount_format: String,
	pub commodity: String,
}

#[derive(Error, Debug)]
pub enum ParserError {
	#[error("IO error: {0}")]
	IoError(String),
	#[error("Format error: {0}")]
	FormatError(String),
}

/// The Ingestion Contract defining how files are converted into standard rows
pub trait Parser {
	fn parse_file(
		&self,
		file_path: &Path,
		template: &ImportTemplate,
	) -> Result<Vec<ParsedRow>, ParserError>;
}
