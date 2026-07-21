use iced::{Element, Length, Size, Task, Theme, widget::container};
use personal::{
	app::Message,
	sankey::{RenderableSankey, SankeyDiagram, compute_layout},
};

pub fn main() -> iced::Result {
	iced::application(
		SankeySandbox::new,
		SankeySandbox::update,
		SankeySandbox::view,
	)
	.subscription(SankeySandbox::subscription)
	.theme(SankeySandbox::theme)
	.run()
}

struct SankeySandbox {
	sankey: SankeyDiagram,
}

#[derive(Debug, Clone)]
enum SandboxMessage {
	SankeyNodeClicked(String),
	LayoutReady(RenderableSankey),
	WindowResized(Size),
}

fn build_sandbox_graph() -> petgraph::Graph<String, f32> {
	let mut graph = petgraph::Graph::new();

	let salary = graph.add_node("Salary".to_string());
	let side_hustle = graph.add_node("Side Hustle".to_string());
	let dividends = graph.add_node("Dividends".to_string());

	let main_checking = graph.add_node("Main Checking".to_string());
	let biz_checking = graph.add_node("Business Checking".to_string());
	let credit_card = graph.add_node("Credit Card".to_string());
	let brokerage = graph.add_node("Brokerage".to_string());

	let taxes = graph.add_node("Taxes".to_string());
	let rent = graph.add_node("Rent".to_string());
	let groceries = graph.add_node("Groceries".to_string());
	let dining = graph.add_node("Dining Out".to_string());
	let travel = graph.add_node("Travel".to_string());
	let subs = graph.add_node("Subscriptions".to_string());
	let savings = graph.add_node("Long-term Savings".to_string());
	let investments = graph.add_node("Stock Purchases".to_string());

	graph.add_edge(salary, main_checking, 10000.0);
	graph.add_edge(side_hustle, biz_checking, 2000.0);
	graph.add_edge(dividends, brokerage, 500.0);

	graph.add_edge(main_checking, taxes, 2000.0);
	graph.add_edge(main_checking, rent, 3000.0);
	graph.add_edge(main_checking, credit_card, 3000.0);
	graph.add_edge(main_checking, savings, 1500.0);
	graph.add_edge(main_checking, brokerage, 500.0);

	graph.add_edge(biz_checking, taxes, 500.0);
	graph.add_edge(biz_checking, savings, 1500.0);

	graph.add_edge(brokerage, investments, 1000.0);

	graph.add_edge(credit_card, groceries, 1000.0);
	graph.add_edge(credit_card, dining, 800.0);
	graph.add_edge(credit_card, travel, 1000.0);
	graph.add_edge(credit_card, subs, 150.0);

	graph
}

impl SankeySandbox {
	fn new() -> (Self, Task<SandboxMessage>) {
		(
			Self {
				sankey: SankeyDiagram::new(RenderableSankey {
					visual_nodes: vec![],
					visual_edges: vec![],
				}),
			},
			Task::none(),
		)
	}

	fn subscription(&self) -> iced::Subscription<SandboxMessage> {
		iced::event::listen_with(|event, _status, _window_id| {
			if let iced::Event::Window(iced::window::Event::Resized(size)) = event {
				Some(SandboxMessage::WindowResized(Size::new(
					size.width as f32,
					size.height as f32,
				)))
			} else {
				None
			}
		})
	}

	fn update(&mut self, message: SandboxMessage) -> Task<SandboxMessage> {
		match message {
			SandboxMessage::SankeyNodeClicked(node_id) => {
				println!("Sandbox: Clicked node: {}", node_id);
				Task::none()
			},
			SandboxMessage::WindowResized(size) => {
				let graph = build_sandbox_graph();
				Task::perform(
					async move {
						let layout =
							compute_layout(&graph, size, &Theme::CatppuccinLatte);
						SandboxMessage::LayoutReady(layout)
					},
					|m| m,
				)
			},
			SandboxMessage::LayoutReady(layout) => {
				self.sankey = SankeyDiagram::new(layout);
				Task::none()
			},
		}
	}

	fn view(&self) -> Element<'_, SandboxMessage> {
		container(self.sankey.view().map(|msg| {
			if let Message::SankeyNodeClicked(node_id) = msg {
				SandboxMessage::SankeyNodeClicked(node_id)
			} else {
				SandboxMessage::SankeyNodeClicked("".to_string())
			}
		}))
		.width(Length::Fill)
		.height(Length::Fill)
		.padding(10)
		.into()
	}

	fn theme(&self) -> Theme {
		Theme::CatppuccinLatte
	}
}
