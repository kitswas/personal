use crate::theme::ElegantTheme;
use egui;

pub struct ElegantAccordion<'a> {
	title: &'a str,
	id_salt: egui::Id,
	default_open: bool,
}

impl<'a> ElegantAccordion<'a> {
	pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, title: &'a str) -> Self {
		Self {
			title,
			id_salt: egui::Id::new(id_salt),
			default_open: false,
		}
	}

	pub fn default_open(mut self, open: bool) -> Self {
		self.default_open = open;
		self
	}

	pub fn show<R>(
		self,
		ui: &mut egui::Ui,
		add_contents: impl FnOnce(&mut egui::Ui) -> R,
	) -> egui::collapsing_header::CollapsingResponse<R> {
		let theme = ElegantTheme::get(ui.ctx());

		// Style the collapsing header to look elegant
		ui.visuals_mut().widgets.inactive.bg_fill = theme.background;
		ui.visuals_mut().widgets.hovered.bg_fill = theme.background.linear_multiply(0.95);
		ui.visuals_mut().widgets.active.bg_fill = theme.background.linear_multiply(0.9);

		ui.visuals_mut().widgets.inactive.fg_stroke.color = theme.foreground;
		ui.visuals_mut().widgets.hovered.fg_stroke.color = theme.foreground;
		ui.visuals_mut().widgets.active.fg_stroke.color = theme.foreground;

		egui::CollapsingHeader::new(self.title)
			.id_salt(self.id_salt)
			.default_open(self.default_open)
			.show(ui, |ui| {
				ui.add_space(8.0);
				add_contents(ui)
			})
	}
}

#[cfg(feature = "flex")]
impl<'a> ElegantAccordion<'a> {
	/// Render this accordion as a flex item inside an [`egui_flex::FlexInstance`].
	///
	/// Returns `CollapsingResponse<R>` which carries both the open/closed state
	/// and the body return value (if the accordion is open).
	pub fn show_flex<R>(
		self,
		flex: &mut egui_flex::FlexInstance,
		item: egui_flex::FlexItem,
		content: impl FnOnce(&mut egui::Ui) -> R,
	) -> egui::collapsing_header::CollapsingResponse<R> {
		flex.add_ui(item, |ui| self.show(ui, content)).inner
	}
}
