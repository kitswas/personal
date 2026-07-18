use egui::{Color32, Context, CornerRadius, Id, Margin, Stroke, Visuals};

#[derive(Clone, Debug)]
pub struct ElegantTheme {
	pub primary: Color32,
	pub secondary: Color32,
	pub background: Color32,
	pub foreground: Color32,
	pub border: Color32,
	pub success: Color32,
}

impl Default for ElegantTheme {
	fn default() -> Self {
		Self {
			primary: Color32::from_rgb(87, 71, 71),
			secondary: Color32::from_rgb(244, 244, 245),
			background: Color32::from_rgb(255, 255, 255),
			foreground: Color32::from_rgb(9, 9, 11),
			border: Color32::from_rgb(212, 212, 216),
			success: Color32::from_rgb(0, 128, 50),
		}
	}
}

impl ElegantTheme {
	pub fn mocha() -> Self {
		Self {
			primary: Color32::from_rgb(203, 166, 247),
			secondary: Color32::from_rgb(49, 50, 68),
			background: Color32::from_rgb(30, 30, 46),
			foreground: Color32::from_rgb(205, 214, 244),
			border: Color32::from_rgb(88, 91, 112),
			success: Color32::from_rgb(166, 227, 161),
		}
	}

	pub fn apply(&self, ctx: &Context) {
		// Store theme in egui memory for components to read
		ctx.data_mut(|d| d.insert_temp(Id::new("elegant_theme"), self.clone()));

		// Update egui's default visuals
		let mut style = (*ctx.style_of(egui::Theme::Light)).clone();

		style.spacing.item_spacing = egui::vec2(16.0, 16.0);
		style.spacing.button_padding = egui::vec2(16.0, 8.0);
		style.spacing.window_margin = Margin::same(24);

		let radius = CornerRadius::same(6);
		style.visuals.window_corner_radius = radius;
		style.visuals.widgets.noninteractive.corner_radius = radius;
		style.visuals.widgets.inactive.corner_radius = radius;
		style.visuals.widgets.hovered.corner_radius = radius;
		style.visuals.widgets.active.corner_radius = radius;

		let mut visuals = if self.background.r() > 128 {
			Visuals::light()
		} else {
			Visuals::dark()
		};
		visuals.window_fill = self.background;
		visuals.panel_fill = self.background;

		visuals.widgets.noninteractive.bg_fill = self.background;
		visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border);
		visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.foreground);

		visuals.widgets.inactive.bg_fill = self.background;
		visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.border);
		visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.foreground);

		visuals.widgets.hovered.bg_fill = self.secondary;
		visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.border);
		visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.foreground);

		visuals.widgets.active.bg_fill = self.border;
		visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.border);
		visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.foreground);

		style.visuals = visuals;

		if self.background.r() > 128 {
			ctx.options_mut(|o| o.theme_preference = egui::ThemePreference::Light);
			ctx.set_style_of(egui::Theme::Light, style);
		} else {
			ctx.options_mut(|o| o.theme_preference = egui::ThemePreference::Dark);
			ctx.set_style_of(egui::Theme::Dark, style);
		}
	}

	pub fn get(ctx: &Context) -> Self {
		ctx.data_mut(|d| {
			d.get_temp(Id::new("elegant_theme"))
				.unwrap_or_else(|| ElegantTheme::default())
		})
	}
}
