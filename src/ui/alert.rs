use eframe::egui;

pub struct Alert<'a> {
    title: &'a str,
    content: &'a str,
}

impl<'a> Alert<'a> {
    pub fn new(title: &'a str, content: &'a str) -> Self {
        Self { title, content }
    }
}

impl<'a> egui::Widget for Alert<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let alert_bg = egui::Color32::from_rgb(244, 244, 245);
        let alert_border = egui::Color32::from_rgb(212, 212, 216);

        egui::Frame::new()
            .fill(alert_bg)
            .stroke(egui::Stroke::new(1.0, alert_border))
            .inner_margin(16.0)
            .corner_radius(egui::CornerRadius::same(6))
            .show(ui, |ui| {
                ui.set_width(300.0);
                ui.label(egui::RichText::new(self.title).strong());
                ui.add_space(4.0);
                ui.label(self.content);
            })
            .response
    }
}
