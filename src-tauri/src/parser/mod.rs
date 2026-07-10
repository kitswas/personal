pub mod csv_parser;
pub mod excel;

use crate::{error::AppError, models::ParsedRow, template::types::ImportTemplate};
use std::path::Path;

/// High-level dispatch for parsing statements based on file extension.
pub fn parse_statement(
	file_path: &Path,
	template: &ImportTemplate,
) -> Result<Vec<ParsedRow>, AppError> {
	let ext = file_path
		.extension()
		.and_then(|s| s.to_str())
		.map(|s| s.to_lowercase())
		.unwrap_or_default();

	match ext.as_str() {
		"csv" => csv_parser::parse_csv(file_path, template),
		"xls" | "xlsx" => excel::parse_excel(file_path, template),
		_ => Err(AppError::Other(format!(
			"Unsupported file extension '{}'. Only CSV and Excel files are supported.",
			ext
		))),
	}
}
