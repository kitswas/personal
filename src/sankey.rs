use crate::app::Message;
use iced::{
	Color, Element, Point, Rectangle, Size, Theme, mouse,
	widget::canvas::{self, Action, Cache, Canvas, Geometry, Program, Stroke},
};
use petgraph::Graph;

pub struct SankeyDiagram {
	cache: Cache,
	graph: Graph<String, f32>,
}

impl SankeyDiagram {
	pub fn new() -> Self {
		let mut graph = Graph::new();
		// Mock data...
		let salary = graph.add_node("Salary".to_string());
		let checking = graph.add_node("Checking Account".to_string());
		let expenses = graph.add_node("Expenses".to_string());

		graph.add_edge(salary, checking, 5000.0);
		graph.add_edge(checking, expenses, 4000.0);

		Self {
			cache: Cache::default(),
			graph,
		}
	}

	pub fn view(&self) -> Element<Message> {
		Canvas::new(self)
			.width(iced::Length::Fill)
			.height(iced::Length::Fill)
			.into()
	}
}

impl Program<Message> for SankeyDiagram {
	type State = ();

	fn draw(
		&self,
		_state: &Self::State,
		renderer: &iced::Renderer,
		_theme: &Theme,
		bounds: Rectangle,
		_cursor: mouse::Cursor,
	) -> Vec<Geometry> {
		let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
			// Very simple mock drawing for Sankey
			let width = frame.width();
			let height = frame.height();

			// Node 1
			let n1_rect = Rectangle::new(
				Point::new(10.0, height / 2.0 - 50.0),
				Size::new(100.0, 100.0),
			);
			frame.fill_rectangle(
				n1_rect.position(),
				n1_rect.size(),
				Color::from_rgb(0.2, 0.5, 0.8),
			);

			// Node 2
			let n2_rect = Rectangle::new(
				Point::new(width - 110.0, height / 2.0 - 50.0),
				Size::new(100.0, 100.0),
			);
			frame.fill_rectangle(
				n2_rect.position(),
				n2_rect.size(),
				Color::from_rgb(0.8, 0.3, 0.3),
			);

			// Edge
			let start =
				Point::new(n1_rect.x + n1_rect.width, n1_rect.y + n1_rect.height / 2.0);
			let end = Point::new(n2_rect.x, n2_rect.y + n2_rect.height / 2.0);

			let mut path = canvas::path::Builder::new();
			path.move_to(start);
			let cp1 = Point::new(start.x + (end.x - start.x) / 2.0, start.y);
			let cp2 = Point::new(start.x + (end.x - start.x) / 2.0, end.y);
			path.bezier_curve_to(cp1, cp2, end);

			frame.stroke(
				&path.build(),
				Stroke::default()
					.with_color(Color::from_rgba(0.5, 0.5, 0.5, 0.5))
					.with_width(20.0),
			);
		});
		vec![geometry]
	}

	fn update(
		&self,
		_state: &mut Self::State,
		event: &iced::Event,
		bounds: Rectangle,
		cursor: mouse::Cursor,
	) -> Option<Action<Message>> {
		if let iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) =
			event
		{
			if let Some(position) = cursor.position_in(bounds) {
				let height = bounds.height;
				let width = bounds.width;

				let n1_rect = Rectangle::new(
					Point::new(10.0, height / 2.0 - 50.0),
					Size::new(100.0, 100.0),
				);
				let n2_rect = Rectangle::new(
					Point::new(width - 110.0, height / 2.0 - 50.0),
					Size::new(100.0, 100.0),
				);

				if n1_rect.contains(position) {
					return Some(Action::publish(Message::SankeyNodeClicked(
						"Left Node".to_string(),
					)));
				}
				if n2_rect.contains(position) {
					return Some(Action::publish(Message::SankeyNodeClicked(
						"Right Node".to_string(),
					)));
				}
			}
		}
		None
	}
}
