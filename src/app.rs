use crate::sankey::SankeyDiagram;
use iced::{
	Element, Length, Task, Theme,
	widget::{button, column, container, row, text},
};

#[derive(Debug, Clone)]
pub enum Message {
	SankeyNodeClicked(String),
	ToggleTheme,
	// other domain messages...
}

pub struct FinanceApp {
	sankey: SankeyDiagram,
	theme: Theme,
}

impl Default for FinanceApp {
	fn default() -> Self {
		Self {
			sankey: SankeyDiagram::new(),
			theme: Theme::Dark, /* Will eventually load from unencrypted local config
			                     * per ADR-0008 */
		}
	}
}

impl FinanceApp {
	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::SankeyNodeClicked(node_id) => {
				println!("Node clicked: {}", node_id);
				Task::none()
			},
			Message::ToggleTheme => {
				self.theme = if self.theme == Theme::Dark {
					Theme::Light
				} else {
					Theme::Dark
				};
				Task::none()
			},
		}
	}

	pub fn view(&self) -> Element<Message> {
		// ADR-0007: Three-pane layout (30-40-30)
		let nav_pane = container(
			column![text("Nav (30%)").size(24), button("Dashboard"),].spacing(16),
		)
		.width(Length::FillPortion(3))
		.padding(24);

		let list_pane = container(
			column![
				text("List (40%)").size(24),
				button("Toggle Theme").on_press(Message::ToggleTheme),
				self.sankey.view(),
			]
			.spacing(16),
		)
		.width(Length::FillPortion(4))
		.padding(24);

		let detail_pane = container(
			column![
				text("Detail (30%)").size(24),
				text("Contextual action surface..."),
			]
			.spacing(16),
		)
		.width(Length::FillPortion(3))
		.padding(24);

		row![nav_pane, list_pane, detail_pane].into()
	}

	pub fn theme(&self) -> Theme {
		self.theme.clone()
	}
}
