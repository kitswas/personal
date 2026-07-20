use crate::domain::parser::{ImportTemplate, ParsedRow, Parser, ParserError};
use calamine::{Data, Reader, open_workbook_auto};
use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::path::Path;

pub struct CsvExcelParser;

impl Parser for CsvExcelParser {
	fn parse_file(
		&self,
		file_path: &Path,
		template: &ImportTemplate,
	) -> Result<Vec<ParsedRow>, ParserError> {
		match template.format.to_lowercase().as_str() {
			"csv" => self.parse_csv(file_path, template),
			"excel" => self.parse_excel(file_path, template),
			other => Err(ParserError::FormatError(format!(
				"Unsupported format: {}",
				other
			))),
		}
	}
}

impl CsvExcelParser {
	fn parse_csv(
		&self,
		file_path: &Path,
		template: &ImportTemplate,
	) -> Result<Vec<ParsedRow>, ParserError> {
		let mut rdr = ReaderBuilder::new()
			.has_headers(template.has_headers)
			.from_path(file_path)
			.map_err(|e| ParserError::IoError(e.to_string()))?;

		let mut rows = Vec::new();

		for (idx, result) in rdr.records().enumerate() {
			let row_idx = if template.has_headers {
				idx + 2
			} else {
				idx + 1
			};
			match result {
				Ok(record) => {
					let record_vec: Vec<&str> = record.iter().collect();
					rows.push(self.process_row(row_idx, &record_vec, template));
				},
				Err(e) => {
					rows.push(ParsedRow::Invalid {
						row_idx,
						raw_data: "Failed to read CSV record".to_string(),
						error_reason: e.to_string(),
					});
				},
			}
		}

		Ok(rows)
	}

	fn parse_excel(
		&self,
		file_path: &Path,
		template: &ImportTemplate,
	) -> Result<Vec<ParsedRow>, ParserError> {
		let mut workbook = open_workbook_auto(file_path)
			.map_err(|e| ParserError::IoError(e.to_string()))?;

		let sheet_names = workbook.sheet_names().to_vec();
		if sheet_names.is_empty() {
			return Err(ParserError::FormatError(
				"No sheets in workbook".to_string(),
			));
		}
		let first_sheet = &sheet_names[0];

		let range = workbook
			.worksheet_range(first_sheet)
			.map_err(|e| ParserError::FormatError(e.to_string()))?;

		let mut rows = Vec::new();
		let mut iter = range.rows().enumerate();

		if template.has_headers {
			iter.next(); // Skip header
		}

		for (idx, row) in iter {
			let row_idx = if template.has_headers {
				idx + 2
			} else {
				idx + 1
			};
			let record_vec: Vec<String> = row
				.iter()
				.map(|d| match d {
					Data::String(s) => s.clone(),
					Data::Float(f) => f.to_string(),
					Data::Int(i) => i.to_string(),
					Data::Bool(b) => b.to_string(),
					Data::DateTime(d) => d.to_string(), /* Keep as float-based string */
					// for now
					Data::DateTimeIso(s) => s.clone(),
					Data::DurationIso(s) => s.clone(),
					Data::Error(e) => format!("Error: {:?}", e),
					Data::Empty => String::new(),
				})
				.collect();

			let refs: Vec<&str> = record_vec.iter().map(|s| s.as_str()).collect();
			rows.push(self.process_row(row_idx, &refs, template));
		}

		Ok(rows)
	}

	fn process_row(
		&self,
		row_idx: usize,
		cols: &[&str],
		template: &ImportTemplate,
	) -> ParsedRow {
		let raw_data = cols.join(",");

		if cols.len() <= template.date_col
			|| cols.len() <= template.payee_col
			|| cols.len() <= template.amount_col
		{
			return ParsedRow::Invalid {
				row_idx,
				raw_data,
				error_reason: "Row does not have enough columns based on template"
					.to_string(),
			};
		}

		let date_str = cols[template.date_col].trim();
		let payee = cols[template.payee_col].trim().to_string();
		let amount_str = cols[template.amount_col].trim();

		// Parse date (Assume standard time T00:00:00Z for CSVs without time)
		let timestamp = match NaiveDate::parse_from_str(date_str, &template.date_format) {
			Ok(d) => format!("{}T00:00:00Z", d.format("%Y-%m-%d")),
			Err(e) => {
				return ParsedRow::Invalid {
					row_idx,
					raw_data,
					error_reason: format!("Date parsing failed: {}", e),
				};
			},
		};

		// Parse amount to paise (integer)
		let amount = match template.amount_format.as_str() {
			"float" => match amount_str.parse::<f64>() {
				Ok(f) => (f * 100.0).round() as i64,
				Err(e) => {
					return ParsedRow::Invalid {
						row_idx,
						raw_data,
						error_reason: format!("Amount parsing failed (float): {}", e),
					};
				},
			},
			"integer" => match amount_str.parse::<i64>() {
				Ok(i) => i * 100,
				Err(e) => {
					return ParsedRow::Invalid {
						row_idx,
						raw_data,
						error_reason: format!("Amount parsing failed (integer): {}", e),
					};
				},
			},
			_ => {
				return ParsedRow::Invalid {
					row_idx,
					raw_data,
					error_reason: format!(
						"Unsupported amount format: {}",
						template.amount_format
					),
				};
			},
		};

		ParsedRow::Valid {
			row_idx,
			timestamp,
			payee,
			amount,
			commodity: template.commodity.clone(),
			suggested_account_id: None,
			confidence: 0.0,
			external_id: None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::{fs::File, io::Write};
	use tempfile::tempdir;

	#[test]
	fn test_csv_parser_valid() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test.csv");
		let mut file = File::create(&file_path).unwrap();
		writeln!(file, "Date,Payee,Amount").unwrap();
		writeln!(file, "2024-01-01,Groceries,-50.25").unwrap();
		writeln!(file, "2024-01-02,Salary,2000.00").unwrap();

		let template = ImportTemplate {
			name: "Test CSV".into(),
			format: "csv".into(),
			has_headers: true,
			date_col: 0,
			date_format: "%Y-%m-%d".into(),
			payee_col: 1,
			amount_col: 2,
			amount_format: "float".into(),
			commodity: "USD".into(),
		};

		let parser = CsvExcelParser;
		let rows = parser.parse_file(&file_path, &template).unwrap();
		assert_eq!(rows.len(), 2);

		match &rows[0] {
			ParsedRow::Valid { payee, amount, .. } => {
				assert_eq!(payee, "Groceries");
				assert_eq!(*amount, -5025);
			},
			_ => panic!("Expected valid row"),
		}

		match &rows[1] {
			ParsedRow::Valid { payee, amount, .. } => {
				assert_eq!(payee, "Salary");
				assert_eq!(*amount, 200000);
			},
			_ => panic!("Expected valid row"),
		}
	}

	#[test]
	fn test_csv_parser_invalid_date() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("bad_date.csv");
		let mut file = File::create(&file_path).unwrap();
		writeln!(file, "not-a-date,Groceries,-50.25").unwrap(); // completely invalid date

		let template = ImportTemplate {
			name: "Test CSV".into(),
			format: "csv".into(),
			has_headers: false,
			date_col: 0,
			date_format: "%Y-%m-%d".into(),
			payee_col: 1,
			amount_col: 2,
			amount_format: "float".into(),
			commodity: "USD".into(),
		};

		let parser = CsvExcelParser;
		let rows = parser.parse_file(&file_path, &template).unwrap();
		assert_eq!(rows.len(), 1);

		match &rows[0] {
			ParsedRow::Invalid { error_reason, .. } => {
				assert!(error_reason.contains("Date parsing failed"));
			},
			_ => panic!("Expected invalid row"),
		}
	}
}
