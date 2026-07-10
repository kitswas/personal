use chrono::NaiveDate;
use std::path::Path;

use crate::{error::AppError, models::ParsedRow, template::types::ImportTemplate};

pub fn parse_csv(
	file_path: &Path,
	template: &ImportTemplate,
) -> Result<Vec<ParsedRow>, AppError> {
	let mut reader = csv::ReaderBuilder::new()
		.has_headers(template.skip_header)
		.flexible(true)
		.from_path(file_path)
		.map_err(|e| AppError::Other(format!("Failed to open CSV: {}", e)))?;

	let mut results = Vec::new();

	for (idx, result) in reader.records().enumerate() {
		// Adjust index if we skipped the header
		let row_idx = if template.skip_header { idx + 1 } else { idx };

		let record = match result {
			Ok(r) => r,
			Err(e) => {
				results.push(ParsedRow::Invalid {
					row_idx,
					raw_data: "".to_string(),
					error_reason: format!("CSV read error: {}", e),
				});
				continue;
			},
		};

		// Join raw data for debugging/display on invalid rows
		let raw_data = record.iter().collect::<Vec<&str>>().join(",");

		let mapping = &template.mapping;
		let max_idx_needed = *[mapping.date, mapping.payee, mapping.amount]
			.iter()
			.max()
			.unwrap();

		if record.len() <= max_idx_needed {
			results.push(ParsedRow::Invalid {
				row_idx,
				raw_data,
				error_reason: format!(
					"Row has {} columns, but mapping requires at least index {}",
					record.len(),
					max_idx_needed
				),
			});
			continue;
		}

		let date_str = record.get(mapping.date).unwrap_or("").trim();
		let payee_str = record.get(mapping.payee).unwrap_or("").trim();
		let amount_str = record.get(mapping.amount).unwrap_or("").trim();

		// Parse date
		let parsed_date = match NaiveDate::parse_from_str(date_str, &mapping.date_format)
		{
			Ok(d) => d,
			Err(e) => {
				results.push(ParsedRow::Invalid {
					row_idx,
					raw_data,
					error_reason: format!("Invalid date format '{}': {}", date_str, e),
				});
				continue;
			},
		};

		// Parse amount (expecting standard float string, converting to smallest unit)
		// e.g. "123.45" -> 12345
		let amount_float: f64 = match amount_str.replace(',', "").parse() {
			Ok(f) => f,
			Err(e) => {
				results.push(ParsedRow::Invalid {
					row_idx,
					raw_data,
					error_reason: format!(
						"Invalid amount format '{}': {}",
						amount_str, e
					),
				});
				continue;
			},
		};

		let mut amount_cents = (amount_float * 100.0).round() as i64;
		if mapping.invert_amount {
			amount_cents = -amount_cents;
		}

		results.push(ParsedRow::Valid {
			row_idx,
			date: parsed_date.format("%Y-%m-%d").to_string(),
			payee: payee_str.to_string(),
			amount: amount_cents,
			commodity: mapping.commodity.clone(),
			suggested_account_id: String::new(),
			confidence: 0.0,
		});
	}

	Ok(results)
}
