use crate::app::Message;
use iced::{
	Element, Point, Rectangle, Size, Theme, mouse,
	widget::canvas::{self, Action, Cache, Canvas, Geometry, Program, Stroke, Text},
};
use petgraph::{Direction, Graph, visit::EdgeRef};
use std::collections::HashMap;

struct NodeLayout {
	rect: Rectangle,
	label: String,
}

fn compute_layout(
	graph: &Graph<String, f32>,
	bounds_size: Size,
) -> (Vec<NodeLayout>, Vec<(Point, Point, Point, Point, f32)>) {
	let mut ranks: HashMap<petgraph::graph::NodeIndex, usize> = HashMap::new();
	let mut changed = true;
	while changed {
		changed = false;
		for nx in graph.node_indices() {
			let mut max_parent_rank = 0;
			let mut has_parents = false;
			for edge in graph.edges_directed(nx, Direction::Incoming) {
				has_parents = true;
				let px = edge.source();
				let pr = ranks.get(&px).copied().unwrap_or(0);
				if pr + 1 > max_parent_rank {
					max_parent_rank = pr + 1;
				}
			}
			let new_rank = if has_parents { max_parent_rank } else { 0 };
			if ranks.get(&nx).copied().unwrap_or(usize::MAX) != new_rank {
				ranks.insert(nx, new_rank);
				changed = true;
			}
		}
	}

	let max_rank = ranks.values().copied().max().unwrap_or(0);
	let num_ranks = max_rank + 1;

	let mut node_flows = HashMap::new();
	let mut rank_flows = vec![0.0; num_ranks];

	for nx in graph.node_indices() {
		let in_flow: f32 = graph
			.edges_directed(nx, Direction::Incoming)
			.map(|e| e.weight())
			.sum();
		let out_flow: f32 = graph
			.edges_directed(nx, Direction::Outgoing)
			.map(|e| e.weight())
			.sum();
		let total_flow = in_flow.max(out_flow).max(1.0);
		node_flows.insert(nx, total_flow);

		let r = ranks[&nx];
		rank_flows[r] += total_flow;
	}

	let max_rank_flow = rank_flows.iter().copied().fold(0.0, f32::max).max(1.0);

	let padding_y = 20.0;
	let node_width = 40.0;

	let available_width = bounds_size.width - node_width;
	let col_spacing = if num_ranks > 1 {
		available_width / (num_ranks - 1) as f32
	} else {
		0.0
	};

	let available_height = bounds_size.height - padding_y * 4.0;
	let pixels_per_flow = available_height / max_rank_flow;

	let mut node_layouts = HashMap::new();
	let mut current_y_per_rank = vec![padding_y; num_ranks];

	for r in 0..num_ranks {
		for nx in graph.node_indices() {
			if ranks[&nx] == r {
				let x = r as f32 * col_spacing;
				let y = current_y_per_rank[r];
				let flow = node_flows[&nx];
				let height = flow * pixels_per_flow;

				let rect =
					Rectangle::new(Point::new(x, y), Size::new(node_width, height));
				node_layouts.insert(
					nx,
					NodeLayout {
						rect,
						label: graph[nx].clone(),
					},
				);

				current_y_per_rank[r] += height + padding_y;
			}
		}
	}

	let mut links = Vec::new();
	let mut current_out_y = HashMap::new();
	let mut current_in_y = HashMap::new();

	for edge in graph.edge_indices() {
		if let Some((src, dst)) = graph.edge_endpoints(edge) {
			let weight = graph[edge];
			let thickness = weight * pixels_per_flow;

			let src_rect = &node_layouts[&src].rect;
			let dst_rect = &node_layouts[&dst].rect;

			let out_offset = current_out_y.entry(src).or_insert(0.0);
			let in_offset = current_in_y.entry(dst).or_insert(0.0);

			let start_x = src_rect.x + src_rect.width;
			let start_y = src_rect.y + *out_offset + thickness / 2.0;

			let end_x = dst_rect.x;
			let end_y = dst_rect.y + *in_offset + thickness / 2.0;

			let cp1 = Point::new(start_x + (end_x - start_x) / 2.0, start_y);
			let cp2 = Point::new(start_x + (end_x - start_x) / 2.0, end_y);

			links.push((
				Point::new(start_x, start_y),
				cp1,
				cp2,
				Point::new(end_x, end_y),
				thickness,
			));

			*out_offset += thickness;
			*in_offset += thickness;
		}
	}

	let node_layout_list = node_layouts.into_values().collect();
	(node_layout_list, links)
}

pub struct SankeyDiagram {
	cache: Cache,
	graph: Graph<String, f32>,
}

impl From<Graph<String, f32>> for SankeyDiagram {
	fn from(graph: Graph<String, f32>) -> Self {
		Self {
			cache: Cache::default(),
			graph,
		}
	}
}

impl SankeyDiagram {

	pub fn view(&self) -> Element<'_, Message> {
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
		theme: &Theme,
		bounds: Rectangle,
		_cursor: mouse::Cursor,
	) -> Vec<Geometry> {
		let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
			let (nodes, links) = compute_layout(&self.graph, frame.size());

			let link_color = iced::Color {
				a: 0.2,
				..theme.palette().text
			};

			for (start, cp1, cp2, end, thickness) in links {
				let mut path = canvas::path::Builder::new();
				path.move_to(start);
				path.bezier_curve_to(cp1, cp2, end);

				frame.stroke(
					&path.build(),
					Stroke::default()
						.with_color(link_color)
						.with_width(thickness),
				);
			}

			for node in nodes {
				frame.fill_rectangle(
					node.rect.position(),
					node.rect.size(),
					theme.palette().primary,
				);

				frame.fill_text(Text {
					content: node.label.clone(),
					position: Point::new(node.rect.x, node.rect.y - 15.0),
					color: theme.palette().text,
					size: 14.0.into(),
					..Default::default()
				});
			}
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
				let (nodes, _) = compute_layout(&self.graph, bounds.size());

				for node in nodes {
					if node.rect.contains(position) {
						return Some(Action::publish(Message::SankeyNodeClicked(
							node.label,
						)));
					}
				}
			}
		}
		None
	}
}
