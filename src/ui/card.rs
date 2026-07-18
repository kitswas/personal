use eframe::egui;
use crate::ui::button::ButtonUiExtensions;

pub struct Card<'a> {
    title: &'a str,
    content: &'a str,
}

impl<'a> Card<'a> {
    pub fn new(title: &'a str, content: &'a str) -> Self {
        Self { title, content }
    }
}

impl<'a> egui::Widget for Card<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(212, 212, 216))) // border
            .inner_margin(24.0)
            .corner_radius(egui::CornerRadius::same(6))
            .show(ui, |ui| {
                ui.set_width(300.0);
                ui.heading(self.title);
                ui.add_space(8.0);
                ui.label(self.content);
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.ghost_button("Cancel");
                    ui.primary_button("Save Changes");
                });
            })
            .response
    }
}
