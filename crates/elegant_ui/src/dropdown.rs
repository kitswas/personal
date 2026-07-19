use crate::theme::ElegantTheme;
use egui;

pub struct ElegantDropdown<'a, T> {
	id_salt: egui::Id,
	selected: &'a mut T,
	options: Vec<(T, String)>,
}

impl<'a, T: PartialEq + Clone> ElegantDropdown<'a, T> {
	pub fn new(
		id_salt: impl std::hash::Hash + std::fmt::Debug,
		selected: &'a mut T,
	) -> Self {
		Self {
			id_salt: egui::Id::new(id_salt),
			selected,
			options: Vec::new(),
		}
	}

	pub fn options(mut self, options: impl IntoIterator<Item = (T, String)>) -> Self {
		self.options = options.into_iter().collect();
		self
	}

	pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
		let theme = ElegantTheme::get(ui.ctx());

		ui.visuals_mut().widgets.inactive.bg_fill =
			theme.background.linear_multiply(0.95);
		ui.visuals_mut().widgets.hovered.bg_fill = theme.background.linear_multiply(0.9);
		ui.visuals_mut().widgets.active.bg_fill = theme.background.linear_multiply(0.85);

		ui.visuals_mut().widgets.inactive.bg_stroke =
			egui::Stroke::new(1.0, theme.foreground.linear_multiply(0.1));
		ui.visuals_mut().widgets.hovered.bg_stroke =
			egui::Stroke::new(1.0, theme.primary);
		ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::new(1.0, theme.primary);

		let mut selected_text = "Select...".to_string();
		for (val, text) in &self.options {
			if val == self.selected {
				selected_text = text.clone();
				break;
			}
		}

		egui::ComboBox::from_id_salt(self.id_salt)
			.selected_text(selected_text)
			.show_ui(ui, |ui| {
				for (val, text) in self.options {
					ui.selectable_value(self.selected, val, text);
				}
			})
			.response
	}
}

#[cfg(feature = "flex")]
impl<'a, T: PartialEq + Clone> ElegantDropdown<'a, T> {
	/// Render this dropdown as a flex item inside an [`egui_flex::FlexInstance`].
	pub fn show_flex(
		self,
		flex: &mut egui_flex::FlexInstance,
		item: egui_flex::FlexItem,
	) -> egui::Response {
		flex.add_ui(item, |ui| self.show(ui)).inner
	}
}
