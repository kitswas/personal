use crate::{
	domain::{
		ledger::Ledger,
		models::{Account, AccountType},
		storage::Storage,
	},
	infrastructure::{core_ledger::CoreLedger, sqlite_storage::SqliteStorage},
	sankey::SankeyDiagram,
};
use iced::{
	Element, Length, Task, Theme,
	widget::{button, column, container, row, text_input},
};
use iced_selection::text;
use std::{
	path::PathBuf,
	sync::Arc,
	time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct Toast {
	pub message: String,
	pub expires_at: Instant,
}

#[derive(Debug, Clone)]
pub enum OperationState {
	Idle,
	Loading(String),
	Triage(Vec<crate::domain::parser::ParsedRow>),
	Success(String),
	Error(String),
}

#[derive(Debug, Clone, Default)]
pub enum OnboardingPhase {
	#[default]
	Input,
	Submitting,
	Error(String),
}

#[derive(Debug, Clone)]
pub struct OnboardingState {
	pub password: String,
	pub confirm_password: String,
	pub base_commodity: String,
	pub phase: OnboardingPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
	Light,
	Dark,
	#[default]
	System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
	Dashboard,
	Accounts,
	Transactions,
	Import,
	Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
	StorageInitialized(Result<Option<Arc<SqliteStorage>>, String>),
	BalancesLoaded(Result<Vec<(Account, i64)>, String>),
	SankeyNodeClicked(String),
	ChangeThemeMode(ThemeMode),
	CommitTriage,
	TriageCommitted(Result<(), String>),
	ThemeChanged(Theme),
	TabPressed { shift: bool },

	// Onboarding
	OnboardingPasswordChanged(String),
	OnboardingConfirmPasswordChanged(String),
	OnboardingCommodityChanged(String),
	OnboardingSubmit,
	OnboardingComplete(Result<Arc<SqliteStorage>, String>),
	NavigateTo(Route),

	// Global error management
	ErrorOccurred(String),
	DismissToast(usize),
	Tick(Instant),
}

#[derive(Debug, Clone)]
pub struct AppConfig {
	pub theme: Theme,
	pub theme_mode: ThemeMode,
}

pub struct FinanceApp {
	pub config: AppConfig,
	pub state: AppState,
	pub toasts: Vec<Toast>,
}

pub enum AppState {
	Booting,
	Onboarding(OnboardingState),
	Loaded {
		storage: Arc<SqliteStorage>,
		balances: Vec<(Account, i64)>,
		sankey: SankeyDiagram,
		operation: OperationState,
		current_route: Route,
	},
	FatalError(String),
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
							Ok(false) => Ok(None), // Needs onboarding
							Err(e) => Err(e.to_string()),
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

		let app = FinanceApp {
			config: AppConfig {
				theme: cosmic_dark(),
				theme_mode: ThemeMode::System,
			},
			state: AppState::Booting,
			toasts: Vec::new(),
		};
		(app, init_task)
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match &message {
			Message::ErrorOccurred(err) => {
				self.toasts.push(Toast {
					message: err.clone(),
					expires_at: Instant::now() + Duration::from_secs(5),
				});
				return Task::none();
			},
			Message::DismissToast(index) => {
				if *index < self.toasts.len() {
					self.toasts.remove(*index);
				}
				return Task::none();
			},
			Message::Tick(now) => {
				self.toasts.retain(|toast| toast.expires_at > *now);
				return Task::none();
			},
			_ => {},
		}

		if let Message::ChangeThemeMode(mode) = message {
			self.config.theme_mode = mode;
			match mode {
				ThemeMode::Light => self.config.theme = cosmic_light(),
				ThemeMode::Dark => self.config.theme = cosmic_dark(),
				ThemeMode::System => {
					self.config.theme = get_system_theme();
				},
			}
			return Task::none();
		}

		if let Message::ThemeChanged(new_theme) = message {
			self.config.theme = new_theme;
			return Task::none();
		}

		if let Message::TabPressed { shift } = message {
			return if shift {
				iced::widget::operation::focus_previous()
			} else {
				iced::widget::operation::focus_next()
			};
		}

		match &mut self.state {
			AppState::Booting => {
				if let Message::StorageInitialized(res) = message {
					if self.config.theme_mode == ThemeMode::System {
						self.config.theme = get_system_theme();
					}

					match res {
						Ok(Some(storage)) => {
							let storage_clone = Arc::clone(&storage);
							self.state = AppState::Loaded {
								storage,
								balances: Vec::new(),
								sankey: SankeyDiagram::new(
									crate::sankey::RenderableSankey {
										visual_nodes: vec![],
										visual_edges: vec![],
									},
								),
								operation: OperationState::Idle,
								current_route: Route::Dashboard,
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
							self.state = AppState::Onboarding(OnboardingState {
								password: String::new(),
								confirm_password: String::new(),
								base_commodity: "INR".to_string(),
								phase: OnboardingPhase::default(),
							});
						},
						Err(e) => {
							self.state =
								AppState::FatalError(format!("Failed to boot: {}", e));
							return Task::none();
						},
					}
				}
				Task::none()
			},
			AppState::Onboarding(state) => match message {
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
				Message::OnboardingSubmit => {
					let password = state.password.clone();
					if password.is_empty() || password != state.confirm_password {
						state.phase = OnboardingPhase::Error(
							"Passwords do not match or are empty".to_string(),
						);
						return Task::none();
					}
					if state.base_commodity.is_empty() {
						state.phase = OnboardingPhase::Error(
							"Base commodity cannot be empty".to_string(),
						);
						return Task::none();
					}

					let base_commodity = state.base_commodity.clone();
					state.phase = OnboardingPhase::Submitting;

					Task::perform(
						async move {
							let entry =
								keyring::Entry::new("personal_finance_app", "master_key")
									.map_err(|e| e.to_string())?;
							entry.set_password(&password).map_err(|e| e.to_string())?;

							let db_path = PathBuf::from("ledger.db");
							let _ = std::fs::remove_file(&db_path);

							let storage = SqliteStorage::new(db_path, password);
							storage.init_db().map_err(|e| e.to_string())?;
							storage
								.complete_onboarding(&base_commodity)
								.map_err(|e| e.to_string())?;

							Ok(Arc::new(storage))
						},
						Message::OnboardingComplete,
					)
				},
				Message::OnboardingComplete(Ok(storage)) => {
					let storage_clone = Arc::clone(&storage);
					self.state = AppState::Loaded {
						storage,
						balances: Vec::new(),
						sankey: SankeyDiagram::new(crate::sankey::RenderableSankey {
							visual_nodes: vec![],
							visual_edges: vec![],
						}),
						operation: OperationState::Idle,
						current_route: Route::Dashboard,
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
					state.phase =
						OnboardingPhase::Error(format!("Failed to setup DB: {}", e));
					Task::none()
				},
				_ => Task::none(),
			},
			AppState::Loaded {
				storage,
				balances,
				sankey: _,
				operation,
				current_route,
			} => match message {
				Message::NavigateTo(route) => {
					*current_route = route;
					Task::none()
				},
				Message::BalancesLoaded(Ok(b)) => {
					*balances = b;
					Task::none()
				},
				Message::BalancesLoaded(Err(e)) => {
					return self.update(Message::ErrorOccurred(format!(
						"Failed to load balances: {}",
						e
					)));
				},
				Message::SankeyNodeClicked(node_id) => {
					println!("Node clicked: {}", node_id);
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
			AppState::FatalError(_) => Task::none(),
		}
	}

	pub fn view(&self) -> Element<'_, Message> {
		let main_content: Element<'_, Message> = match &self.state {
			AppState::Booting => {
				container(text("Booting & Checking Security...").size(40))
					.width(Length::Fill)
					.height(Length::Fill)
					.center_x(Length::Fill)
					.center_y(Length::Fill)
					.into()
			},
			AppState::FatalError(e) => container(
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
			AppState::Onboarding(state) => match &state.phase {
				OnboardingPhase::Submitting => {
					container(text("Initializing encrypted database...").size(24))
						.width(Length::Fill)
						.height(Length::Fill)
						.center_x(Length::Fill)
						.center_y(Length::Fill)
						.into()
				},
				_ => {
					let mut col = column![
							text("Welcome to Personal Finance").size(40),
							text("Please create a master password. This will encrypt your database at rest with AES-256.").size(16),
							text_input("Master Password", &state.password)
								.id(iced::widget::Id::new("password"))
								.secure(true)
								.on_input(Message::OnboardingPasswordChanged)
								.on_submit(Message::OnboardingSubmit)
								.padding(10),
							text_input("Confirm Password", &state.confirm_password)
								.id(iced::widget::Id::new("confirm_password"))
								.secure(true)
								.on_input(Message::OnboardingConfirmPasswordChanged)
								.on_submit(Message::OnboardingSubmit)
								.padding(10),
							text("Base Commodity (e.g. INR, USD):").size(16),
							text_input("Base Commodity", &state.base_commodity)
								.id(iced::widget::Id::new("commodity"))
								.on_input(Message::OnboardingCommodityChanged)
								.on_submit(Message::OnboardingSubmit)
								.padding(10),
							button("Initialize Database").on_press(Message::OnboardingSubmit).padding(10),
						]
						.spacing(20)
						.max_width(500);

					if let OnboardingPhase::Error(err) = &state.phase {
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
			},
			AppState::Loaded {
				balances,
				sankey,
				operation,
				current_route,
				..
			} => {
				let nav_col = column![
					text("Navigation").size(24),
					button("Dashboard").on_press(Message::NavigateTo(Route::Dashboard)),
					button("Accounts").on_press(Message::NavigateTo(Route::Accounts)),
					button("Transactions")
						.on_press(Message::NavigateTo(Route::Transactions)),
					button("Import").on_press(Message::NavigateTo(Route::Import)),
					button("Settings").on_press(Message::NavigateTo(Route::Settings)),
				]
				.spacing(16);

				let nav_pane =
					container(nav_col).width(Length::FillPortion(3)).padding(24);

				let mut detail_col =
					column![text(format!("Detail: {:?}", current_route)).size(24)]
						.spacing(16);

				match current_route {
					Route::Dashboard => {
						detail_col = detail_col.push(sankey.view());
					},
					Route::Accounts => {
						detail_col = detail_col.push(text("Account Statistics"));
						for (acc, bal) in balances {
							let display_balance = match acc.account_type {
								AccountType::Asset | AccountType::Expense => {
									*bal as f64 / 100.0
								},
								AccountType::Liability
								| AccountType::Equity
								| AccountType::Revenue => -(*bal as f64) / 100.0,
							};
							detail_col = detail_col.push(iced_selection::text(format!(
								"{}: {:.2}",
								acc.name, display_balance
							)));
						}
					},
					Route::Settings => {
						detail_col = detail_col.push(text("Theme Preferences").size(20));
						let modes =
							[ThemeMode::Light, ThemeMode::Dark, ThemeMode::System];
						let mut r = row![].spacing(10);
						for mode in modes {
							r = r.push(iced::widget::radio(
								format!("{:?}", mode),
								mode,
								Some(self.config.theme_mode),
								Message::ChangeThemeMode,
							));
						}
						detail_col = detail_col.push(r);
					},
					_ => {
						detail_col =
							detail_col.push(text("Contextual action surface..."));
					},
				}

				let detail_pane = container(detail_col)
					.width(Length::FillPortion(4))
					.padding(24);

				let mut list_col =
					column![text(format!("List: {:?}", current_route)).size(24),]
						.spacing(16);

				match operation {
					OperationState::Loading(msg) => {
						list_col = list_col.push(text(msg).style(|theme: &Theme| {
							iced_selection::text::Style {
								color: Some(theme.palette().primary),
								..iced_selection::text::default(theme)
							}
						}));
					},
					OperationState::Success(msg) => {
						list_col = list_col.push(text(msg).style(|theme: &Theme| {
							iced_selection::text::Style {
								color: Some(theme.palette().success),
								..iced_selection::text::default(theme)
							}
						}));
					},
					OperationState::Error(msg) => {
						list_col = list_col.push(text(format!("Error: {}", msg)).style(
							|theme: &Theme| iced_selection::text::Style {
								color: Some(theme.palette().danger),
								..iced_selection::text::default(theme)
							},
						));
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
					OperationState::Idle => {},
				}

				let list_pane = container(list_col)
					.width(Length::FillPortion(3))
					.padding(24);

				row![nav_pane, detail_pane, list_pane].into()
			},
		};

		let mut error_stack = column![].spacing(10).width(Length::Fixed(350.0));
		for (index, toast) in self.toasts.iter().enumerate() {
			let toast_row = row![
				text(&toast.message).width(Length::Fill).size(14),
				button("X").on_press(Message::DismissToast(index))
			]
			.align_y(iced::Alignment::Center)
			.spacing(10);

			let toast_box = container(toast_row).padding(12).style(|theme: &Theme| {
				iced::widget::container::Style {
					background: Some(iced::Background::Color(theme.palette().danger)),
					text_color: Some(theme.palette().background), /* use background for
					                                               * contrast */
					border: iced::border::rounded(4),
					..Default::default()
				}
			});

			error_stack = error_stack.push(toast_box);
		}

		let overlay = container(error_stack)
			.width(Length::Fill)
			.height(Length::Fill)
			.align_x(iced::Alignment::End)
			.align_y(iced::Alignment::End)
			.padding(20);

		// Iced 0.14 stack widget
		iced::widget::stack!(main_content, overlay).into()
	}

	pub fn theme(&self) -> Theme {
		self.config.theme.clone()
	}

	pub fn subscription(&self) -> iced::Subscription<Message> {
		let theme_sub = if self.config.theme_mode == ThemeMode::System {
			fn theme_stream() -> impl iced::futures::Stream<Item = Message> {
				iced::stream::channel(
					10,
					|mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
						let _sub = mundy::Preferences::subscribe(
							mundy::Interest::All,
							move |prefs| {
								let theme = resolve_os_theme(prefs);
								println!("MUNDY SENDING THEME: {:?}", prefs.color_scheme);
								if let Err(e) =
									sender.try_send(Message::ThemeChanged(theme))
								{
									println!("SEND FAILED: {:?}", e);
								}
							},
						);
						let () = iced::futures::future::pending().await;
						unreachable!()
					},
				)
			}
			iced::Subscription::run(theme_stream)
		} else {
			iced::Subscription::none()
		};

		let keyboard_sub = iced::event::listen_with(|event, status, _window_id| {
			if status == iced::event::Status::Ignored {
				if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
					key: iced::keyboard::key::Key::Named(iced::keyboard::key::Named::Tab),
					modifiers,
					..
				}) = event
				{
					return Some(Message::TabPressed {
						shift: modifiers.shift(),
					});
				}
			}
			None
		});

		fn toast_stream() -> impl iced::futures::Stream<Item = Message> {
			iced::stream::channel(
				1,
				|mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
					loop {
						tokio::time::sleep(Duration::from_secs(1)).await;
						let _ = sender.try_send(Message::Tick(Instant::now()));
					}
				},
			)
		}

		let toast_sub = if self.toasts.is_empty() {
			iced::Subscription::none()
		} else {
			iced::Subscription::run(toast_stream)
		};

		iced::Subscription::batch([theme_sub, keyboard_sub, toast_sub])
	}
}

pub fn cosmic_light() -> Theme {
	Theme::custom(
		"Cosmic Light".to_string(),
		iced::theme::Palette {
			background: iced::Color::from_rgb8(250, 250, 250),
			text: iced::Color::from_rgb8(24, 24, 27),
			primary: iced::Color::from_rgb8(13, 148, 136),
			success: iced::Color::from_rgb8(16, 185, 129),
			danger: iced::Color::from_rgb8(239, 68, 68),
			warning: iced::Color::from_rgb8(245, 158, 11),
		},
	)
}

pub fn cosmic_dark() -> Theme {
	Theme::custom(
		"Cosmic Dark".to_string(),
		iced::theme::Palette {
			background: iced::Color::from_rgb8(24, 24, 27),
			text: iced::Color::from_rgb8(244, 244, 245),
			primary: iced::Color::from_rgb8(45, 212, 191),
			success: iced::Color::from_rgb8(52, 211, 153),
			danger: iced::Color::from_rgb8(248, 113, 113),
			warning: iced::Color::from_rgb8(251, 191, 36),
		},
	)
}

pub fn resolve_os_theme(prefs: mundy::Preferences) -> Theme {
	match prefs.color_scheme {
		mundy::ColorScheme::Light => cosmic_light(),
		_ => cosmic_dark(),
	}
}

pub fn get_system_theme() -> Theme {
	if let Some(prefs) = mundy::Preferences::once_blocking(
		mundy::Interest::All,
		std::time::Duration::from_millis(50),
	) {
		resolve_os_theme(prefs)
	} else {
		cosmic_dark()
	}
}
