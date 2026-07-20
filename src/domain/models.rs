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
	pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct Transaction {
	pub id: String,
	pub timestamp: String,
	pub payee: String,
	pub notes: Option<String>,
	pub external_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Posting {
	pub id: String,
	pub transaction_id: String,
	pub account_id: String,
	pub amount: i64, // smallest unit (e.g. paise)
	pub commodity: String,
}
