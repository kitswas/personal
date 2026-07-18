use eframe::{
	egui,
	egui::{Color32, Stroke},
};

pub struct FinanceApp {}

// Catppuccin Mocha Palette
const BASE: Color32 = Color32::from_rgb(30, 30, 46);
const MANTLE: Color32 = Color32::from_rgb(24, 24, 37);
const CRUST: Color32 = Color32::from_rgb(17, 17, 27);
const TEXT: Color32 = Color32::from_rgb(205, 214, 244);
const SUBTEXT1: Color32 = Color32::from_rgb(186, 194, 222);
const SUBTEXT0: Color32 = Color32::from_rgb(166, 173, 200);
const SURFACE2: Color32 = Color32::from_rgb(88, 91, 112);
const SURFACE1: Color32 = Color32::from_rgb(69, 71, 90);
const SURFACE0: Color32 = Color32::from_rgb(49, 50, 68);
const SAPPHIRE: Color32 = Color32::from_rgb(116, 199, 236);
const GREEN: Color32 = Color32::from_rgb(166, 227, 161);
// const RED: Color32 = Color32::from_rgb(243, 139, 168);
const MAUVE: Color32 = Color32::from_rgb(203, 166, 247);

impl FinanceApp {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let mut visuals = egui::Visuals::dark();

		// Apply Catppuccin Mocha globally
		visuals.window_fill = MANTLE;
		visuals.panel_fill = BASE;
		visuals.faint_bg_color = SURFACE0;
		visuals.extreme_bg_color = CRUST;

		visuals.widgets.noninteractive.bg_fill = BASE;
		visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
		visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, SURFACE0);

		visuals.widgets.inactive.bg_fill = SURFACE0;
		visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, SUBTEXT1);
		visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, SURFACE1);

		visuals.widgets.hovered.bg_fill = SURFACE1;
		visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
		visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, SURFACE2);

		visuals.widgets.active.bg_fill = SAPPHIRE;
		visuals.widgets.active.fg_stroke = Stroke::new(1.0, BASE);
		visuals.widgets.active.bg_stroke = Stroke::new(1.0, SAPPHIRE);

		visuals.selection.bg_fill = MAUVE;
		visuals.selection.stroke = Stroke::new(1.0, BASE);

		cc.egui_ctx.set_visuals(visuals);

		Self {}
	}
}

impl eframe::App for FinanceApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		// Adjust spacing for a more airy, modern interface
		ui.spacing_mut().item_spacing = egui::vec2(16.0, 16.0);
		ui.spacing_mut().button_padding = egui::vec2(16.0, 8.0);

		egui::Frame::new().fill(BASE).show(ui, |ui| {
			// Top Header bar
			egui::Frame::new()
				.fill(CRUST)
				.inner_margin(16.0)
				.show(ui, |ui| {
					ui.horizontal(|ui| {
						let title = egui::RichText::new("Antigravity Personal Finance")
							.size(20.0)
							.strong()
							.color(MAUVE);
						ui.heading(title);

						ui.with_layout(
							egui::Layout::right_to_left(egui::Align::Center),
							|ui| {
								ui.label(
									egui::RichText::new("Secure Local Vault")
										.color(SUBTEXT0),
								);
								ui.add(egui::Spinner::new().color(MAUVE));
							},
						);
					});
				});

			// Main layout: Sidebar + Content
			ui.horizontal(|ui| {
				// Sidebar
				egui::Frame::new()
					.fill(MANTLE)
					.inner_margin(16.0)
					.show(ui, |ui| {
						ui.set_width(220.0);
						ui.set_min_height(800.0); // take up space

						ui.add_space(20.0);
						ui.vertical_centered_justified(|ui| {
							let _ = ui.selectable_label(
								true,
								egui::RichText::new("⛶ Dashboard").size(16.0).color(TEXT),
							);
							ui.add_space(8.0);
							let _ = ui.selectable_label(
								false,
								egui::RichText::new("📥 Import Data")
									.size(16.0)
									.color(SUBTEXT1),
							);
							ui.add_space(8.0);
							let _ = ui.selectable_label(
								false,
								egui::RichText::new("💳 Accounts")
									.size(16.0)
									.color(SUBTEXT1),
							);
							ui.add_space(8.0);
							let _ = ui.selectable_label(
								false,
								egui::RichText::new("⚙ Settings")
									.size(16.0)
									.color(SUBTEXT1),
							);
						});
					});

				// Main Content Area
				egui::Frame::new()
					.fill(BASE)
					.inner_margin(32.0)
					.show(ui, |ui| {
						ui.label(
							egui::RichText::new("Dashboard")
								.size(36.0)
								.strong()
								.color(TEXT),
						);
						ui.add_space(8.0);
						ui.label(
							egui::RichText::new(
								"Welcome back! Here's your financial overview.",
							)
							.color(SUBTEXT0),
						);
						ui.add_space(32.0);

						ui.horizontal(|ui| {
							// Card: Net Worth
							egui::Frame::group(ui.style())
								.fill(SURFACE0)
								.stroke(Stroke::new(1.0, SURFACE1))
								.inner_margin(24.0)
								.show(ui, |ui| {
									ui.set_width(240.0);
									ui.label(
										egui::RichText::new("Net Worth")
											.size(16.0)
											.color(SUBTEXT1),
									);
									ui.add_space(4.0);
									ui.label(
										egui::RichText::new("₹ 0.00")
											.size(32.0)
											.strong()
											.color(SAPPHIRE),
									);
								});

							ui.add_space(16.0);

							// Card: Liquid Assets
							egui::Frame::group(ui.style())
								.fill(SURFACE0)
								.stroke(Stroke::new(1.0, SURFACE1))
								.inner_margin(24.0)
								.show(ui, |ui| {
									ui.set_width(240.0);
									ui.label(
										egui::RichText::new("Liquid Assets")
											.size(16.0)
											.color(SUBTEXT1),
									);
									ui.add_space(4.0);
									ui.label(
										egui::RichText::new("₹ 0.00")
											.size(32.0)
											.strong()
											.color(GREEN),
									);
								});
						});

						ui.add_space(48.0);
						ui.label(
							egui::RichText::new("Recent Transactions")
								.size(24.0)
								.strong()
								.color(TEXT),
						);
						ui.add_space(24.0);

						// Empty State Area
						egui::Frame::group(ui.style())
							.fill(MANTLE)
							.stroke(Stroke::new(1.0, SURFACE0))
							.inner_margin(48.0)
							.show(ui, |ui| {
								ui.vertical_centered(|ui| {
									ui.label(
										egui::RichText::new("No transactions yet.")
											.size(18.0)
											.color(SUBTEXT0),
									);
									ui.add_space(16.0);
									if ui
										.button(
											egui::RichText::new(
												"Import your first statement",
											)
											.size(16.0)
											.color(BASE),
										)
										.clicked()
									{
										// handle click
									}
								});
							});
					});
			});
		});
	}
}
