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

		// Income Sources
		let salary = graph.add_node("Salary".to_string());
		let side_hustle = graph.add_node("Side Hustle".to_string());
		let dividends = graph.add_node("Dividends".to_string());

		// Intermediate Accounts
		let main_checking = graph.add_node("Main Checking".to_string());
		let biz_checking = graph.add_node("Business Checking".to_string());
		let credit_card = graph.add_node("Credit Card".to_string());
		let brokerage = graph.add_node("Brokerage".to_string());

		// Sinks / Expenses / Savings
		let taxes = graph.add_node("Taxes".to_string());
		let rent = graph.add_node("Rent".to_string());
		let groceries = graph.add_node("Groceries".to_string());
		let dining = graph.add_node("Dining Out".to_string());
		let travel = graph.add_node("Travel".to_string());
		let subs = graph.add_node("Subscriptions".to_string());
		let savings = graph.add_node("Long-term Savings".to_string());
		let investments = graph.add_node("Stock Purchases".to_string());

		// Edges from Income
		graph.add_edge(salary, main_checking, 10000.0);
		graph.add_edge(side_hustle, biz_checking, 2000.0);
		graph.add_edge(dividends, brokerage, 500.0);

		// Edges from Main Checking (Total In: 10000, Total Out: 10000 -> Balanced)
		graph.add_edge(main_checking, taxes, 2000.0);
		graph.add_edge(main_checking, rent, 3000.0);
		graph.add_edge(main_checking, credit_card, 3000.0);
		graph.add_edge(main_checking, savings, 1500.0);
		graph.add_edge(main_checking, brokerage, 500.0);

		// Edges from Business Checking (Total In: 2000, Total Out: 2000 -> Balanced)
		graph.add_edge(biz_checking, taxes, 500.0);
		graph.add_edge(biz_checking, savings, 1500.0);

		// Edges from Brokerage (Total In: 1000, Total Out: 1000 -> Balanced)
		graph.add_edge(brokerage, investments, 1000.0);

		// Edges from Credit Card (Total In: 3000, Total Out: 2950 -> UNBALANCED!)
		graph.add_edge(credit_card, groceries, 1000.0);
		graph.add_edge(credit_card, dining, 800.0);
		graph.add_edge(credit_card, travel, 1000.0);
		graph.add_edge(credit_card, subs, 150.0);
		// 50.0 is missing to show the danger color check

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
