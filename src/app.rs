use crate::{
	domain::{
		ledger::Ledger,
		models::{Account, AccountType},
		parser::{ImportTemplate, Parser},
		storage::Storage,
	},
	infrastructure::{
		core_ledger::CoreLedger, csv_excel_parser::CsvExcelParser,
		sqlite_storage::SqliteStorage,
	},
	sankey::SankeyDiagram,
};
use iced::{
	Element, Length, Task, Theme,
	widget::{button, checkbox, column, container, row, text_input},
};
use iced_selection::text;
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone)]
pub enum OperationState {
	Idle,
	Loading(String),
	Triage(Vec<crate::domain::parser::ParsedRow>),
	Success(String),
	Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct OnboardingState {
	pub password: String,
	pub confirm_password: String,
	pub base_commodity: String,
	pub create_seed_accounts: bool,
	pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
	StorageInitialized(Result<Option<Arc<SqliteStorage>>, String>),
	BalancesLoaded(Result<Vec<(Account, i64)>, String>),
	SankeyNodeClicked(String),
	ToggleTheme,
	LoadTestData,
	TestDataLoaded(Result<Vec<crate::domain::parser::ParsedRow>, String>),
	CommitTriage,
	TriageCommitted(Result<(), String>),
	ThemeChanged(Theme),

	// Onboarding
	OnboardingPasswordChanged(String),
	OnboardingConfirmPasswordChanged(String),
	OnboardingCommodityChanged(String),
	OnboardingToggleSeedAccounts(bool),
	OnboardingSubmit,
	OnboardingComplete(Result<Arc<SqliteStorage>, String>),
}

pub enum FinanceApp {
	Booting,
	Onboarding(OnboardingState),
	Loaded {
		storage: Arc<SqliteStorage>,
		balances: Vec<(Account, i64)>,
		sankey: SankeyDiagram,
		theme: Theme,
		operation: OperationState,
	},
	Error(String),
}

impl FinanceApp {
	pub fn new() -> (Self, Task<Message>) {
		let init_task = Task::perform(
			async {
				// We wrap in a block to isolate keyring errors
				let entry = keyring::Entry::new("personal_finance_app", "master_key")
					.map_err(|e| format!("Keyring error: {}", e))?;

				let db_path = PathBuf::from("ledger.db");

				match entry.get_password() {
					Ok(password) => {
						let storage = SqliteStorage::new(db_path, password);
						// Check if DB exists and is fully initialized
						match storage.is_onboarding_done() {
							Ok(true) => Ok(Some(Arc::new(storage))),
							_ => Ok(None), // Needs onboarding
						}
					},
					Err(_) => {
						// Password not found in keyring
						Ok(None)
					},
				}
			},
			Message::StorageInitialized,
		);

		(FinanceApp::Booting, init_task)
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match self {
			FinanceApp::Booting => {
				if let Message::StorageInitialized(res) = message {
					match res {
						Ok(Some(storage)) => {
							let storage_clone = Arc::clone(&storage);
							*self = FinanceApp::Loaded {
								storage,
								balances: Vec::new(),
								sankey: SankeyDiagram::new(),
								theme: cosmic_dark(),
								operation: OperationState::Idle,
							};
							return Task::perform(
								async move {
									match storage_clone.get_running_balances() {
										Ok(b) => Ok(b),
										Err(e) => Err(e.to_string()),
									}
								},
								Message::BalancesLoaded,
							);
						},
						Ok(None) => {
							let mut state = OnboardingState::default();
							state.base_commodity = "INR".to_string();
							state.create_seed_accounts = true;
							*self = FinanceApp::Onboarding(state);
						},
						Err(e) => {
							*self = FinanceApp::Error(format!("Failed to boot: {}", e));
						},
					}
				}
				Task::none()
			},
			FinanceApp::Onboarding(state) => {
				match message {
					Message::OnboardingPasswordChanged(val) => {
						state.password = val;
						Task::none()
					},
					Message::OnboardingConfirmPasswordChanged(val) => {
						state.confirm_password = val;
						Task::none()
					},
					Message::OnboardingCommodityChanged(val) => {
						state.base_commodity = val;
						Task::none()
					},
					Message::OnboardingToggleSeedAccounts(val) => {
						state.create_seed_accounts = val;
						Task::none()
					},
					Message::OnboardingSubmit => {
						let password = state.password.clone();
						if password.is_empty() || password != state.confirm_password {
							state.error =
								Some("Passwords do not match or are empty".to_string());
							return Task::none();
						}
						if state.base_commodity.is_empty() {
							state.error =
								Some("Base commodity cannot be empty".to_string());
							return Task::none();
						}

						let base_commodity = state.base_commodity.clone();
						let seed = state.create_seed_accounts;

						Task::perform(
							async move {
								let entry = keyring::Entry::new(
									"personal_finance_app",
									"master_key",
								)
								.map_err(|e| e.to_string())?;
								entry
									.set_password(&password)
									.map_err(|e| e.to_string())?;

								let db_path = PathBuf::from("ledger.db");
								// Delete the DB to ensure we start from a clean state
								// with the new password
								let _ = std::fs::remove_file(&db_path);

								let storage = SqliteStorage::new(db_path, password);
								storage.init_db().map_err(|e| e.to_string())?;
								storage
									.complete_onboarding(&base_commodity, seed)
									.map_err(|e| e.to_string())?;

								Ok(Arc::new(storage))
							},
							Message::OnboardingComplete,
						)
					},
					Message::OnboardingComplete(Ok(storage)) => {
						let storage_clone = Arc::clone(&storage);
						*self = FinanceApp::Loaded {
							storage,
							balances: Vec::new(),
							sankey: SankeyDiagram::new(),
							theme: cosmic_dark(),
							operation: OperationState::Idle,
						};
						Task::perform(
							async move {
								match storage_clone.get_running_balances() {
									Ok(b) => Ok(b),
									Err(e) => Err(e.to_string()),
								}
							},
							Message::BalancesLoaded,
						)
					},
					Message::OnboardingComplete(Err(e)) => {
						state.error = Some(format!("Failed to setup DB: {}", e));
						Task::none()
					},
					_ => Task::none(),
				}
			},
			FinanceApp::Loaded {
				storage,
				balances,
				sankey: _,
				theme,
				operation,
			} => match message {
				Message::BalancesLoaded(Ok(b)) => {
					*balances = b;
					Task::none()
				},
				Message::BalancesLoaded(Err(e)) => {
					*self = FinanceApp::Error(format!("Failed to load balances: {}", e));
					Task::none()
				},
				Message::SankeyNodeClicked(node_id) => {
					println!("Node clicked: {}", node_id);
					Task::none()
				},
				Message::ToggleTheme => {
					*theme = if *theme == cosmic_dark() {
						cosmic_light()
					} else {
						cosmic_dark()
					};
					Task::none()
				},
				Message::ThemeChanged(new_theme) => {
					*theme = new_theme;
					Task::none()
				},
				Message::LoadTestData => {
					*operation = OperationState::Loading("Importing CSV Data...".into());
					let storage_clone = Arc::clone(storage);

					Task::perform(
						async move {
							// 0. Ensure target accounts exist
							storage_clone
								.save_account(&Account {
									id: "assets:bank".into(),
									name: "Checking Account".into(),
									account_type: AccountType::Asset,
									commodity: "INR".into(),
									is_active: true,
								})
								.map_err(|e| e.to_string())?;
							storage_clone
								.save_account(&Account {
									id: "expenses:food".into(),
									name: "Groceries".into(),
									account_type: AccountType::Expense,
									commodity: "INR".into(),
									is_active: true,
								})
								.map_err(|e| e.to_string())?;
							storage_clone
								.save_account(&Account {
									id: "revenue:salary".into(),
									name: "Salary".into(),
									account_type: AccountType::Revenue,
									commodity: "INR".into(),
									is_active: true,
								})
								.map_err(|e| e.to_string())?;

							// 1. Create dummy CSV
							let csv_path = PathBuf::from("test_data.csv");
							std::fs::write(
								&csv_path,
								"Date,Payee,Amount\n2024-01-01,Groceries,-120.50\n2024-01-02,Salary,3000.00\n",
							)
							.map_err(|e| e.to_string())?;

							let template = ImportTemplate {
								name: "Test CSV".into(),
								format: "csv".into(),
								has_headers: true,
								date_col: 0,
								date_format: "%Y-%m-%d".into(),
								payee_col: 1,
								amount_col: 2,
								amount_format: "float".into(),
								commodity: "INR".into(),
							};

							// 2. Parse using CsvExcelParser
							let parser = CsvExcelParser;
							let mut rows = parser
								.parse_file(&csv_path, &template)
								.map_err(|e| e.to_string())?;

							// 3. Auto-Categorize
							let mut categorizer =
								crate::domain::categorizer::Categorizer::new();
							categorizer
								.add_rule(r"(?i)groceries", "expenses:food")
								.unwrap();
							categorizer
								.add_rule(r"(?i)salary", "revenue:salary")
								.unwrap();

							for row in &mut rows {
								if let crate::domain::parser::ParsedRow::Valid {
									suggested_account_id,
									confidence,
									payee,
									..
								} = row
								{
									if let Some((acc, conf)) =
										categorizer.categorize(payee)
									{
										*suggested_account_id = Some(acc);
										*confidence = conf;
									}
								}
							}

							Ok(rows)
						},
						Message::TestDataLoaded,
					)
				},
				Message::TestDataLoaded(Ok(rows)) => {
					*operation = OperationState::Triage(rows);
					Task::none()
				},
				Message::TestDataLoaded(Err(e)) => {
					*operation = OperationState::Error(e);
					Task::none()
				},
				Message::CommitTriage => {
					if let OperationState::Triage(rows) = operation {
						let rows_clone = rows.clone();
						let storage_clone = Arc::clone(storage);
						*operation =
							OperationState::Loading("Committing Transactions...".into());
						Task::perform(
							async move {
								let ledger = CoreLedger;
								ledger
									.validate_and_commit(
										&rows_clone,
										storage_clone.as_ref(),
									)
									.map_err(|e| e.to_string())
							},
							Message::TriageCommitted,
						)
					} else {
						Task::none()
					}
				},
				Message::TriageCommitted(Ok(_)) => {
					*operation = OperationState::Success(
						"Transactions committed successfully!".into(),
					);
					let storage_clone = Arc::clone(storage);
					Task::perform(
						async move {
							match storage_clone.get_running_balances() {
								Ok(b) => Ok(b),
								Err(e) => Err(e.to_string()),
							}
						},
						Message::BalancesLoaded,
					)
				},
				Message::TriageCommitted(Err(e)) => {
					*operation = OperationState::Error(e);
					Task::none()
				},
				_ => Task::none(),
			},
			FinanceApp::Error(_) => Task::none(),
		}
	}

	pub fn view(&self) -> Element<Message> {
		match self {
			FinanceApp::Booting => {
				container(text("Booting & Checking Security...").size(40))
					.width(Length::Fill)
					.height(Length::Fill)
					.center_x(Length::Fill)
					.center_y(Length::Fill)
					.into()
			},
			FinanceApp::Onboarding(state) => {
				let mut col = column![
					text("Welcome to Personal Finance").size(40),
					text("Please create a master password. This will encrypt your database at rest with AES-256.").size(16),
					text_input("Master Password", &state.password)
						.secure(true)
						.on_input(Message::OnboardingPasswordChanged)
						.padding(10),
					text_input("Confirm Password", &state.confirm_password)
						.secure(true)
						.on_input(Message::OnboardingConfirmPasswordChanged)
						.padding(10),
					text("Base Commodity (e.g. INR, USD):").size(16),
					text_input("Base Commodity", &state.base_commodity)
						.on_input(Message::OnboardingCommodityChanged)
						.padding(10),
					checkbox(state.create_seed_accounts)
						.label("Create default seed accounts")
					.on_toggle(Message::OnboardingToggleSeedAccounts),
					button("Initialize Database").on_press(Message::OnboardingSubmit).padding(10),
				]
				.spacing(20)
				.max_width(500);

				if let Some(err) = &state.error {
					col = col.push(text(err).style(|theme: &Theme| {
						iced_selection::text::Style {
							color: Some(theme.palette().danger),
							..iced_selection::text::default(theme)
						}
					}));
				}

				container(col)
					.width(Length::Fill)
					.height(Length::Fill)
					.center_x(Length::Fill)
					.center_y(Length::Fill)
					.into()
			},
			FinanceApp::Error(e) => container(
				column![
					text("Fatal Error").size(40).style(|theme: &Theme| {
						iced_selection::text::Style {
							color: Some(theme.palette().danger),
							..iced_selection::text::default(theme)
						}
					}),
					text(e).size(20),
				]
				.spacing(20)
				.align_x(iced::Alignment::Center),
			)
			.width(Length::Fill)
			.height(Length::Fill)
			.center_x(Length::Fill)
			.center_y(Length::Fill)
			.into(),
			FinanceApp::Loaded {
				balances,
				sankey,
				operation,
				..
			} => {
				// ADR-0007: Three-pane layout (30-40-30)

				// Build the Nav Pane
				let mut nav_col = column![text("Accounts").size(24)].spacing(16);

				if balances.is_empty() {
					nav_col = nav_col.push(text("No accounts or balances found."));
				} else {
					for (acc, bal) in balances {
						let display_balance = match acc.account_type {
							AccountType::Asset | AccountType::Expense => {
								*bal as f64 / 100.0
							},
							AccountType::Liability
							| AccountType::Equity
							| AccountType::Revenue => -(*bal as f64) / 100.0,
						};
						nav_col = nav_col.push(iced_selection::text(format!(
							"{}: {:.2}",
							acc.name, display_balance
						)));
					}
				}

				let nav_pane =
					container(nav_col).width(Length::FillPortion(3)).padding(24);

				// List Pane
				let mut list_col = column![
					text("Overview (40%)").size(24),
					row![
						button("Toggle Theme").on_press(Message::ToggleTheme),
						button("Load Test CSV").on_press(Message::LoadTestData)
					]
					.spacing(16),
				]
				.spacing(16);

				match operation {
					OperationState::Idle => {
						list_col = list_col.push(sankey.view());
					},
					OperationState::Loading(msg) => {
						list_col = list_col.push(text(msg).style(|theme: &Theme| {
							iced_selection::text::Style {
								color: Some(theme.palette().primary),
								..iced_selection::text::default(theme)
							}
						}));
						list_col = list_col.push(sankey.view());
					},
					OperationState::Success(msg) => {
						list_col = list_col.push(text(msg).style(|theme: &Theme| {
							iced_selection::text::Style {
								color: Some(theme.palette().success),
								..iced_selection::text::default(theme)
							}
						}));
						list_col = list_col.push(sankey.view());
					},
					OperationState::Error(msg) => {
						list_col = list_col.push(text(format!("Error: {}", msg)).style(
							|theme: &Theme| iced_selection::text::Style {
								color: Some(theme.palette().danger),
								..iced_selection::text::default(theme)
							},
						));
						list_col = list_col.push(sankey.view());
					},
					OperationState::Triage(rows) => {
						let mut triage_col =
							column![text("Triage Import Data").size(20)].spacing(10);
						for row_item in rows {
							match row_item {
								crate::domain::parser::ParsedRow::Valid {
									payee,
									amount,
									suggested_account_id,
									confidence,
									..
								} => {
									let acc_text = suggested_account_id
										.as_deref()
										.unwrap_or("UNKNOWN");
									let is_high_conf = *confidence > 0.0;
									triage_col = triage_col.push(
										row![
											text(payee).width(Length::FillPortion(2)),
											text(format!(
												"{:.2}",
												*amount as f64 / 100.0
											))
											.width(Length::FillPortion(1)),
											text(format!(
												"{} ({}%)",
												acc_text, confidence
											))
											.style(move |t: &Theme| {
												iced_selection::text::Style {
													color: Some(if is_high_conf {
														t.palette().success
													} else {
														t.palette().danger
													}),
													..iced_selection::text::default(t)
												}
											})
											.width(Length::FillPortion(2))
										]
										.spacing(10),
									);
								},
								crate::domain::parser::ParsedRow::Invalid {
									raw_data,
									error_reason,
									..
								} => {
									triage_col = triage_col.push(
										text(format!(
											"INVALID: {} - {}",
											raw_data, error_reason
										))
										.style(|t: &Theme| {
											iced_selection::text::Style {
												color: Some(t.palette().danger),
												..iced_selection::text::default(t)
											}
										}),
									);
								},
							}
						}
						triage_col = triage_col.push(
							button("Commit Transactions").on_press(Message::CommitTriage),
						);
						list_col = list_col.push(triage_col);
					},
				}

				let list_pane = container(list_col)
					.width(Length::FillPortion(4))
					.padding(24);

				// Detail Pane
				let detail_pane = container(
					column![
						text("Detail (30%)").size(24),
						text("Contextual action surface..."),
					]
					.spacing(16),
				)
				.width(Length::FillPortion(3))
				.padding(24);

				row![nav_pane, list_pane, detail_pane].into()
			},
		}
	}

	pub fn theme(&self) -> Theme {
		match self {
			FinanceApp::Loaded { theme, .. } => theme.clone(),
			_ => cosmic_dark(),
		}
	}

	pub fn subscription(&self) -> iced::Subscription<Message> {
		fn theme_stream() -> impl iced::futures::Stream<Item = Message> {
			tokio_stream::StreamExt::map(
				mundy::Preferences::stream(mundy::Interest::ColorScheme),
				|prefs| {
					let theme = match prefs.color_scheme {
						mundy::ColorScheme::Dark => cosmic_dark(),
						mundy::ColorScheme::Light => cosmic_light(),
						_ => cosmic_dark(),
					};
					Message::ThemeChanged(theme)
				},
			)
		}

		iced::Subscription::run(theme_stream)
	}
}

pub fn cosmic_light() -> Theme {
	Theme::custom(
		"Cosmic Light".to_string(),
		iced::theme::Palette {
			background: iced::Color::from_rgb(0.957, 0.957, 0.961),
			text: iced::Color::from_rgb(0.141, 0.141, 0.141),
			primary: iced::Color::from_rgb(0.282, 0.725, 0.780),
			success: iced::Color::from_rgb(0.957, 0.545, 0.161),
			danger: iced::Color::from_rgb(0.898, 0.224, 0.208),
			warning: iced::Color::from_rgb(0.957, 0.745, 0.161),
		},
	)
}

pub fn cosmic_dark() -> Theme {
	Theme::custom(
		"Cosmic Dark".to_string(),
		iced::theme::Palette {
			background: iced::Color::from_rgb(0.141, 0.141, 0.141),
			text: iced::Color::from_rgb(0.957, 0.957, 0.961),
			primary: iced::Color::from_rgb(0.282, 0.725, 0.780),
			success: iced::Color::from_rgb(0.957, 0.545, 0.161),
			danger: iced::Color::from_rgb(0.898, 0.224, 0.208),
			warning: iced::Color::from_rgb(0.957, 0.745, 0.161),
		},
	)
}
