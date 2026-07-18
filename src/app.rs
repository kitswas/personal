use crate::state::{AppState, Command, Message};
use crossbeam_channel::{Receiver, Sender};
use eframe::egui;

pub struct FinanceApp {
	state: AppState,
	msg_sender: Sender<Message>,
	msg_receiver: Receiver<Message>,
}

impl FinanceApp {
	pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
		let (msg_sender, msg_receiver) = crossbeam_channel::unbounded();
		Self {
			state: AppState::new(),
			msg_sender,
			msg_receiver,
		}
	}

	fn handle_command(&self, cmd: Command, ctx: egui::Context) {
		match cmd {
			Command::None => {},
			Command::LoadData => {
				let sender = self.msg_sender.clone();
				tokio::spawn(async move {
					// Simulate async work
					tokio::time::sleep(std::time::Duration::from_millis(500)).await;
					let result = Ok(15000.50);
					let _ = sender.send(Message::DataLoaded(result));
					ctx.request_repaint();
				});
			},
		}
	}
}

impl eframe::App for FinanceApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		// 1. Drain messages and apply state transitions
		while let Ok(msg) = self.msg_receiver.try_recv() {
			let cmd = self.state.apply_message(msg);
			self.handle_command(cmd, ui.ctx().clone());
		}

		// 2. Render UI strictly from &AppState
		render_ui(ui, &self.state, self.msg_sender.clone());
	}
}

// Strictly pure view projection
fn render_ui(ui: &mut egui::Ui, state: &AppState, sender: Sender<Message>) {
	ui.heading("Local-First Personal Finance");

	ui.horizontal(|ui| {
		ui.label("Total Balance:");
		ui.label(format!("${:.2}", state.total_balance));
		if ui.button("Refresh Data").clicked() {
			let _ = sender.send(Message::FetchData);
		}
	});

	ui.separator();

	ui.heading("Mock Interactive Sankey");

	// Mock Interactive Sankey Node
	let node_id = "Housing".to_string();
	let is_expanded = state.expanded_nodes.contains(&node_id);

	let (rect, response) =
		ui.allocate_exact_size(egui::vec2(150.0, 50.0), egui::Sense::click());

	// Interactivity
	if response.clicked() {
		let _ = sender.send(Message::ToggleSankeyNode(node_id.clone()));
	}

	// Tooltip on hover
	response.on_hover_text(format!("Category: {}\nClick to toggle expansion", node_id));

	// Drawing
	let color = if is_expanded {
		egui::Color32::from_rgb(100, 150, 250)
	} else {
		egui::Color32::from_rgb(150, 150, 150)
	};

	ui.painter().rect_filled(rect, 5.0, color);

	let text_color = egui::Color32::WHITE;
	let font_id = egui::FontId::proportional(16.0);
	let galley = ui.painter().layout_no_wrap(node_id, font_id, text_color);
	let text_pos = egui::pos2(
		rect.center().x - galley.size().x / 2.0,
		rect.center().y - galley.size().y / 2.0,
	);
	ui.painter().galley(text_pos, galley, text_color);
}
