use iced::{Element, Length, Task, Theme, widget::container};
use personal::{app::Message, sankey::SankeyDiagram};

pub fn main() -> iced::Result {
	iced::application(
		SankeySandbox::new,
		SankeySandbox::update,
		SankeySandbox::view,
	)
	.theme(SankeySandbox::theme)
	.run()
}

struct SankeySandbox {
	sankey: SankeyDiagram,
}

impl SankeySandbox {
	fn new() -> (Self, Task<Message>) {
		(
			Self {
				sankey: SankeyDiagram::new(),
			},
			Task::none(),
		)
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		if let Message::SankeyNodeClicked(node_id) = message {
			println!("Sandbox: Clicked node: {}", node_id);
		}
		Task::none()
	}

	fn view(&self) -> Element<'_, Message> {
		container(self.sankey.view())
			.width(Length::Fill)
			.height(Length::Fill)
			.padding(40)
			.into()
	}

	fn theme(&self) -> Theme {
		personal::app::cosmic_dark()
	}
}
