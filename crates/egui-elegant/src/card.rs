use crate::theme::ElegantTheme;
use egui;

#[derive(Default)]
pub struct Card {}

impl Card {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn show<R>(
		self,
		ui: &mut egui::Ui,
		add_contents: impl FnOnce(&mut egui::Ui) -> R,
	) -> egui::InnerResponse<R> {
		let theme = ElegantTheme::get(ui.ctx());

		egui::Frame::new()
            .fill(theme.secondary.linear_multiply(0.3)) // faint background
            .stroke(egui::Stroke::new(theme.spacing.border_width, theme.border))
            .inner_margin(theme.spacing.card_inner_margin)
            .corner_radius(egui::CornerRadius::same(theme.spacing.corner_radius as u8))
            .show(ui, add_contents)
	}
}

#[cfg(feature = "flex")]
impl Card {
	/// Render this card as a flex item inside an [`egui_flex::FlexInstance`].
	///
	/// Equivalent to `flex.add_ui(item, |ui| self.show(ui, content).inner)`.
	pub fn show_flex<R>(
		self,
		flex: &mut egui_flex::FlexInstance,
		item: egui_flex::FlexItem,
		content: impl FnOnce(&mut egui::Ui) -> R,
	) -> egui::InnerResponse<R> {
		flex.add_ui(item, |ui| self.show(ui, content).inner)
	}
}
