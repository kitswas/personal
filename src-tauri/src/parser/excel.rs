use calamine::{DataType, Reader, Xlsx, open_workbook};
use chrono::{Datelike, NaiveDate};
use std::path::Path;

use crate::{error::AppError, models::ParsedRow, template::types::ImportTemplate};

pub fn parse_excel(
	file_path: &Path,
	template: &ImportTemplate,
) -> Result<Vec<ParsedRow>, AppError> {
	let mut workbook: Xlsx<_> = open_workbook(file_path)
		.map_err(|e| AppError::Other(format!("Failed to open Excel file: {}", e)))?;

	let sheet_names = workbook.sheet_names().to_owned();
	if sheet_names.is_empty() {
		return Err(AppError::Other("Excel workbook has no sheets".into()));
	}

	let first_sheet = &sheet_names[0];
	let range = workbook
		.worksheet_range(first_sheet)
		.map_err(|e| AppError::Other(format!("Excel error: {}", e)))?;

	let mut results = Vec::new();

	let rows = range.rows();
	let mut iter = rows.enumerate();

	if template.skip_header {
		iter.next();
	}

	let mapping = &template.mapping;
	let max_idx_needed = *[mapping.date, mapping.payee, mapping.amount]
		.iter()
		.max()
		.unwrap();

	for (idx, row) in iter {
		let row_idx = idx;

		let raw_data = row
			.iter()
			.map(|d| d.to_string())
			.collect::<Vec<String>>()
			.join(",");

		if row.len() <= max_idx_needed {
			results.push(ParsedRow::Invalid {
				row_idx,
				raw_data,
				error_reason: format!(
					"Row has {} columns, but mapping requires at least index {}",
					row.len(),
					max_idx_needed
				),
			});
			continue;
		}

		let date_cell = &row[mapping.date];
		let payee_cell = &row[mapping.payee];
		let amount_cell = &row[mapping.amount];

		let payee_str = payee_cell.to_string().trim().to_string();
		let amount_str = amount_cell.to_string().trim().to_string();

		// Handle date
		let parsed_date = match date_cell {
			calamine::Data::Float(_) | calamine::Data::Int(_) => {
				// Excel dates are floating point days since 1899-12-30 (typically)
				// For simplicity we try string parsing first, but calamine provides as_date
				if let Some(d) = date_cell.as_date() {
					NaiveDate::from_ymd_opt(d.year(), d.month(), d.day())
						.ok_or_else(|| "Invalid date".to_string())
				} else {
					Err("Could not convert Excel number to date".to_string())
				}
			},
			calamine::Data::String(s) => {
				NaiveDate::parse_from_str(s, &mapping.date_format)
					.map_err(|e| format!("Invalid date format '{}': {}", s, e))
			},
			_ => Err(format!("Unrecognized date format: {}", date_cell)),
		};

		let parsed_date = match parsed_date {
			Ok(d) => d,
			Err(e) => {
				results.push(ParsedRow::Invalid {
					row_idx,
					raw_data,
					error_reason: e,
				});
				continue;
			},
		};

		// Handle amount
		let amount_float: f64 = match amount_cell.get_float() {
			Some(f) => f,
			None => match amount_cell.get_int() {
				Some(i) => i as f64,
				None => match amount_str.replace(',', "").parse() {
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
				},
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
