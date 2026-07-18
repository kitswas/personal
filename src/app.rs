use eframe::{
	egui,
	egui::{Color32, Stroke},
};

pub struct FinanceApp {}

impl FinanceApp {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let mut visuals = egui::Visuals::dark();

		visuals.window_fill = Color32::from_rgb(15, 23, 42);
		visuals.panel_fill = Color32::from_rgb(15, 23, 42);
		visuals.faint_bg_color = Color32::from_rgb(30, 41, 59);
		visuals.extreme_bg_color = Color32::from_rgb(2, 6, 23);

		visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 41, 59);
		visuals.widgets.noninteractive.bg_stroke =
			Stroke::new(1.0, Color32::from_rgb(51, 65, 85));
		visuals.widgets.noninteractive.fg_stroke =
			Stroke::new(1.0, Color32::from_rgb(241, 245, 249));

		visuals.widgets.inactive.bg_fill = Color32::from_rgb(51, 65, 85);
		visuals.widgets.inactive.fg_stroke =
			Stroke::new(1.0, Color32::from_rgb(248, 250, 252));

		visuals.widgets.hovered.bg_fill = Color32::from_rgb(71, 85, 105);
		visuals.widgets.hovered.fg_stroke =
			Stroke::new(1.0, Color32::from_rgb(255, 255, 255));

		visuals.widgets.active.bg_fill = Color32::from_rgb(14, 165, 233);
		visuals.widgets.active.fg_stroke =
			Stroke::new(1.0, Color32::from_rgb(255, 255, 255));

		cc.egui_ctx.set_visuals(visuals);
		Self {}
	}
}

impl eframe::App for FinanceApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		let slate_100 = Color32::from_rgb(241, 245, 249);
		let slate_400 = Color32::from_rgb(148, 163, 184);
		let slate_800 = Color32::from_rgb(30, 41, 59);
		let sky_500 = Color32::from_rgb(14, 165, 233);
		let green_500 = Color32::from_rgb(34, 197, 94);

		ui.horizontal(|ui| {
			ui.heading(
				egui::RichText::new("Antigravity Personal Finance").color(slate_100),
			);
			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				ui.label(egui::RichText::new("Secure Local Vault").color(slate_400));
				ui.add(egui::Spinner::new());
			});
		});

		ui.separator();

		ui.horizontal(|ui| {
			// Sidebar
			ui.vertical(|ui| {
				ui.set_width(150.0);
				ui.add_space(20.0);
				let _ = ui.selectable_label(true, "Dashboard");
				ui.add_space(8.0);
				let _ = ui.selectable_label(false, "Import Data");
				ui.add_space(8.0);
				let _ = ui.selectable_label(false, "Accounts");
				ui.add_space(8.0);
				let _ = ui.selectable_label(false, "Settings");
			});

			ui.separator();

			// Main Content
			ui.vertical(|ui| {
				ui.add_space(10.0);
				ui.label(egui::RichText::new("Dashboard").size(32.0).color(slate_100));
				ui.add_space(16.0);

				ui.horizontal(|ui| {
					egui::Frame::group(ui.style())
						.fill(slate_800)
						.inner_margin(16.0)
						.show(ui, |ui| {
							ui.set_width(200.0);
							ui.label(egui::RichText::new("Net Worth").color(slate_400));
							ui.label(
								egui::RichText::new("₹ 0.00").size(28.0).color(sky_500),
							);
						});

					ui.add_space(16.0);

					egui::Frame::group(ui.style())
						.fill(slate_800)
						.inner_margin(16.0)
						.show(ui, |ui| {
							ui.set_width(200.0);
							ui.label(
								egui::RichText::new("Liquid Assets").color(slate_400),
							);
							ui.label(
								egui::RichText::new("₹ 0.00").size(28.0).color(green_500),
							);
						});
				});

				ui.add_space(32.0);
				ui.label(
					egui::RichText::new("Recent Transactions")
						.size(24.0)
						.color(slate_100),
				);
				ui.add_space(16.0);

				egui::Frame::group(ui.style())
					.fill(slate_800)
					.inner_margin(32.0)
					.show(ui, |ui| {
						ui.vertical_centered(|ui| {
							ui.label(
								egui::RichText::new("No transactions yet.")
									.color(slate_400),
							);
							ui.add_space(8.0);
							if ui.button("Import Data").clicked() {
								// handle click
							}
						});
					});
			});
		});
	}
}
