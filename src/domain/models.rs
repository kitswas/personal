#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountType {
	Asset,
	Liability,
	Equity,
	Revenue,
	Expense,
}

#[derive(Debug, Clone)]
pub struct Account {
	pub id: String,
	pub name: String,
	pub account_type: AccountType,
	pub commodity: String,
}

#[derive(Debug, Clone)]
pub struct Transaction {
	pub id: String,
	pub date: String, // ISO 8601
	pub payee: String,
	pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Posting {
	pub id: String,
	pub transaction_id: String,
	pub account_id: String,
	pub amount: i64, // smallest unit (e.g. paise)
	pub commodity: String,
}
