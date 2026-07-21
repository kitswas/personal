use crate::app::Message;
use iced::{
	Element, Point, Rectangle, Size, Theme, mouse,
	widget::canvas::{self, Action, Cache, Canvas, Geometry, Program, Stroke, Text},
};
use petgraph::{Direction, Graph, visit::EdgeRef};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RenderableSankey {
	pub visual_nodes: Vec<RenderNode>,
	pub visual_edges: Vec<RenderEdge>,
}

#[derive(Clone, Debug)]
pub struct RenderNode {
	pub bounds: Rectangle,
	pub color: iced::Color,
	pub label: String,
	pub is_balanced: bool,
}

#[derive(Clone, Debug)]
pub struct RenderEdge {
	pub source_point: Point,
	pub target_point: Point,
	pub flow_thickness: f32,
	pub control_point_1: Point,
	pub control_point_2: Point,
}

impl RenderEdge {
	pub fn build_bezier_path(&self) -> canvas::Path {
		canvas::Path::new(|builder| {
			builder.move_to(self.source_point);
			builder.bezier_curve_to(
				self.control_point_1,
				self.control_point_2,
				self.target_point,
			);
		})
	}
}

pub fn compute_layout(
	graph: &Graph<String, f32>,
	bounds_size: Size,
	theme: &Theme,
) -> RenderableSankey {
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
	let mut node_balanced = HashMap::new();
	let mut rank_flows = vec![0.0; num_ranks];

	for nx in graph.node_indices() {
		let mut has_in = false;
		let in_flow: f32 = graph
			.edges_directed(nx, Direction::Incoming)
			.map(|e| {
				has_in = true;
				e.weight()
			})
			.sum();
		let mut has_out = false;
		let out_flow: f32 = graph
			.edges_directed(nx, Direction::Outgoing)
			.map(|e| {
				has_out = true;
				e.weight()
			})
			.sum();
		let total_flow = in_flow.max(out_flow).max(1.0);
		node_flows.insert(nx, total_flow);

		let is_balanced = if has_in && has_out {
			(in_flow - out_flow).abs() < f32::EPSILON
		} else {
			true
		};
		node_balanced.insert(nx, is_balanced);

		let r = ranks[&nx];
		rank_flows[r] += total_flow;
	}

	let max_rank_flow = rank_flows.iter().copied().fold(0.0, f32::max).max(1.0);

	let padding_y = 20.0;
	let padding_x = 150.0; // Margin for rightmost text labels
	let node_width = 40.0;

	let available_width = bounds_size.width - node_width - padding_x;
	let col_spacing = if num_ranks > 1 {
		available_width / (num_ranks - 1) as f32
	} else {
		0.0
	};

	let mut max_nodes_in_rank = 0;
	let mut nodes_in_rank = vec![0; num_ranks];
	for nx in graph.node_indices() {
		let r = ranks[&nx];
		nodes_in_rank[r] += 1;
		if nodes_in_rank[r] > max_nodes_in_rank {
			max_nodes_in_rank = nodes_in_rank[r];
		}
	}

	let total_padding_y = (max_nodes_in_rank as f32 + 1.0) * padding_y;
	let available_height = (bounds_size.height - total_padding_y).max(1.0);
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

				let node_color = if node_balanced[&nx] {
					theme.palette().primary
				} else {
					theme.palette().danger
				};

				let rect =
					Rectangle::new(Point::new(x, y), Size::new(node_width, height));
				node_layouts.insert(
					nx,
					RenderNode {
						bounds: rect,
						color: node_color,
						label: graph[nx].clone(),
						is_balanced: node_balanced[&nx],
					},
				);

				current_y_per_rank[r] += height + padding_y;
			}
		}
	}

	let mut links = Vec::new();

	let mut outgoing_edges = HashMap::new();
	let mut incoming_edges = HashMap::new();

	for edge in graph.edge_indices() {
		if let Some((src, dst)) = graph.edge_endpoints(edge) {
			outgoing_edges
				.entry(src)
				.or_insert_with(Vec::new)
				.push(edge);
			incoming_edges
				.entry(dst)
				.or_insert_with(Vec::new)
				.push(edge);
		}
	}

	let mut edge_out_offset = HashMap::new();
	for (_, mut edges) in outgoing_edges {
		edges.sort_by(|&e1, &e2| {
			let d1_opt = graph.edge_endpoints(e1);
			let d2_opt = graph.edge_endpoints(e2);
			if let (Some((_, d1)), Some((_, d2))) = (d1_opt, d2_opt) {
				let y1 = node_layouts.get(&d1).map(|n| n.bounds.y).unwrap_or(0.0);
				let y2 = node_layouts.get(&d2).map(|n| n.bounds.y).unwrap_or(0.0);
				y1.partial_cmp(&y2).unwrap_or(std::cmp::Ordering::Equal)
			} else {
				std::cmp::Ordering::Equal
			}
		});
		let mut current_y = 0.0;
		for e in edges {
			edge_out_offset.insert(e, current_y);
			current_y += graph[e] * pixels_per_flow;
		}
	}

	let mut edge_in_offset = HashMap::new();
	for (_, mut edges) in incoming_edges {
		edges.sort_by(|&e1, &e2| {
			let s1_opt = graph.edge_endpoints(e1);
			let s2_opt = graph.edge_endpoints(e2);
			if let (Some((s1, _)), Some((s2, _))) = (s1_opt, s2_opt) {
				let y1 = node_layouts.get(&s1).map(|n| n.bounds.y).unwrap_or(0.0);
				let y2 = node_layouts.get(&s2).map(|n| n.bounds.y).unwrap_or(0.0);
				y1.partial_cmp(&y2).unwrap_or(std::cmp::Ordering::Equal)
			} else {
				std::cmp::Ordering::Equal
			}
		});
		let mut current_y = 0.0;
		for e in edges {
			edge_in_offset.insert(e, current_y);
			current_y += graph[e] * pixels_per_flow;
		}
	}

	for edge in graph.edge_indices() {
		if let Some((src, dst)) = graph.edge_endpoints(edge) {
			let weight = graph[edge];
			let thickness = weight * pixels_per_flow;

			let src_rect = &node_layouts[&src].bounds;
			let dst_rect = &node_layouts[&dst].bounds;

			let out_offset = edge_out_offset[&edge];
			let in_offset = edge_in_offset[&edge];

			let start_x = src_rect.x + src_rect.width;
			let start_y = src_rect.y + out_offset + thickness / 2.0;

			let end_x = dst_rect.x;
			let end_y = dst_rect.y + in_offset + thickness / 2.0;

			let cp1 = Point::new(start_x + (end_x - start_x) / 2.0, start_y);
			let cp2 = Point::new(start_x + (end_x - start_x) / 2.0, end_y);

			links.push(RenderEdge {
				source_point: Point::new(start_x, start_y),
				target_point: Point::new(end_x, end_y),
				flow_thickness: thickness,
				control_point_1: cp1,
				control_point_2: cp2,
			});
		}
	}

	RenderableSankey {
		visual_nodes: node_layouts.into_values().collect(),
		visual_edges: links,
	}
}

pub struct SankeyDiagram {
	pub cache: Cache,
	pub layout_data: RenderableSankey,
}

impl SankeyDiagram {
	pub fn new(layout_data: RenderableSankey) -> Self {
		Self {
			cache: Cache::default(),
			layout_data,
		}
	}

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
			let link_color = iced::Color {
				a: 0.2,
				..theme.palette().text
			};

			for edge in &self.layout_data.visual_edges {
				let path = edge.build_bezier_path();

				frame.stroke(
					&path,
					Stroke::default()
						.with_color(link_color)
						.with_width(edge.flow_thickness),
				);
			}

			for node in &self.layout_data.visual_nodes {
				frame.fill_rectangle(
					node.bounds.position(),
					node.bounds.size(),
					node.color,
				);

				frame.fill_text(Text {
					content: node.label.clone(),
					position: Point::new(node.bounds.x, node.bounds.y - 15.0),
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
				for node in &self.layout_data.visual_nodes {
					if node.bounds.contains(position) {
						return Some(Action::publish(Message::SankeyNodeClicked(
							node.label.clone(),
						)));
					}
				}
			}
		}
		None
	}
}
