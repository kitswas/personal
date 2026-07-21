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
		let mut graph = petgraph::Graph::new();
		// Mock data...
		let salary = graph.add_node("Salary".to_string());
		let checking = graph.add_node("Checking Account".to_string());
		let expenses = graph.add_node("Expenses".to_string());

		graph.add_edge(salary, checking, 5000.0);
		graph.add_edge(checking, expenses, 4000.0);
		(
			Self {
				sankey: SankeyDiagram::from(graph),
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
		Theme::CatppuccinLatte
	}
}
