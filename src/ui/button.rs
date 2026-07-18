use eframe::egui;

pub trait ButtonUiExtensions {
    fn primary_button(&mut self, text: &str) -> egui::Response;
    fn ghost_button(&mut self, text: &str) -> egui::Response;
}

impl ButtonUiExtensions for egui::Ui {
    fn primary_button(&mut self, text: &str) -> egui::Response {
        let btn = egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE))
            .fill(egui::Color32::from_rgb(87, 71, 71)) // primary color
            .stroke(egui::Stroke::NONE);
        self.add(btn)
    }

    fn ghost_button(&mut self, text: &str) -> egui::Response {
        let btn = egui::Button::new(text)
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE);
        self.add(btn)
    }
}
