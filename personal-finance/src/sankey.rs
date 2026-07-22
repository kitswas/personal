use crate::app::Message;
use iced::{
	Element, Point, Rectangle, Size, Theme, mouse,
	widget::canvas::{Cache, Canvas, Geometry, Path, Program, Stroke, Text},
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

pub struct SankeyDiagram {
	cache: Cache,
	pub data: RenderableSankey,
}

impl SankeyDiagram {
	pub fn new(data: RenderableSankey) -> Self {
		Self {
			cache: Cache::new(),
			data,
		}
	}

	pub fn view(&self) -> Element<'_, Message> {
		Canvas::new(self)
			.width(iced::Length::Fill)
			.height(iced::Length::Fill)
			.into()
	}

	pub fn compute_layout(
		graph: &Graph<String, f32>,
		node_colors: &HashMap<String, iced::Color>,
		_viewport_width: f32,
		viewport_height: f32,
	) -> RenderableSankey {
		let mut visual_nodes = Vec::new();
		let mut visual_edges = Vec::new();

		let node_count = graph.node_count();
		if node_count == 0 {
			return RenderableSankey {
				visual_nodes,
				visual_edges,
			};
		}

		let node_width = 30.0;
		let layer_padding = 200.0;

		let mut layer_map: HashMap<petgraph::graph::NodeIndex, usize> = HashMap::new();
		for node in graph.node_indices() {
			let in_degree = graph.edges_directed(node, Direction::Incoming).count();
			if in_degree == 0 {
				layer_map.insert(node, 0);
			} else {
				layer_map.insert(node, 1);
			}
		}

		let max_layer = layer_map.values().max().copied().unwrap_or(0);
		let mut layers: Vec<Vec<petgraph::graph::NodeIndex>> =
			vec![Vec::new(); max_layer + 1];

		for (node, layer) in &layer_map {
			layers[*layer].push(*node);
		}

		let mut node_bounds: HashMap<petgraph::graph::NodeIndex, Rectangle> =
			HashMap::new();

		for (layer_idx, nodes) in layers.iter().enumerate() {
			let x = 50.0 + (layer_idx as f32 * (node_width + layer_padding));
			let total_height = viewport_height - 100.0;
			let step_y = total_height / (nodes.len() as f32 + 1.0);

			for (node_idx_in_layer, &node_idx) in nodes.iter().enumerate() {
				let y = 50.0 + ((node_idx_in_layer as f32 + 1.0) * step_y);
				let height = 40.0;

				let rect = Rectangle {
					x,
					y,
					width: node_width,
					height,
				};
				node_bounds.insert(node_idx, rect);

				let label = graph[node_idx].clone();
				let color = node_colors
					.get(&label)
					.copied()
					.unwrap_or(iced::Color::from_rgb(0.5, 0.5, 0.5));

				visual_nodes.push(RenderNode {
					bounds: rect,
					color,
					label,
					is_balanced: true,
				});
			}
		}

		for edge in graph.edge_references() {
			let source = edge.source();
			let target = edge.target();
			let weight = *edge.weight();

			if let (Some(source_rect), Some(target_rect)) =
				(node_bounds.get(&source), node_bounds.get(&target))
			{
				let source_point = Point::new(
					source_rect.x + source_rect.width,
					source_rect.y + (source_rect.height / 2.0),
				);
				let target_point =
					Point::new(target_rect.x, target_rect.y + (target_rect.height / 2.0));

				let dx = target_point.x - source_point.x;
				let control_point_1 =
					Point::new(source_point.x + (dx * 0.5), source_point.y);
				let control_point_2 =
					Point::new(source_point.x + (dx * 0.5), target_point.y);

				visual_edges.push(RenderEdge {
					source_point,
					target_point,
					flow_thickness: weight.max(2.0),
					control_point_1,
					control_point_2,
				});
			}
		}

		RenderableSankey {
			visual_nodes,
			visual_edges,
		}
	}
}

impl<Message> Program<Message> for SankeyDiagram
where
	Message: Clone,
{
	type State = ();

	fn draw(
		&self,
		_state: &Self::State,
		renderer: &iced::Renderer,
		_theme: &Theme,
		bounds: Rectangle,
		_cursor: mouse::Cursor,
	) -> Vec<Geometry> {
		let geom = self.cache.draw(renderer, bounds.size(), |frame| {
			for edge in &self.data.visual_edges {
				let path = Path::new(|b| {
					b.move_to(edge.source_point);
					b.bezier_curve_to(
						edge.control_point_1,
						edge.control_point_2,
						edge.target_point,
					);
				});

				frame.stroke(
					&path,
					Stroke::default()
						.with_color(iced::Color::from_rgba(0.2, 0.6, 1.0, 0.4))
						.with_width(edge.flow_thickness),
				);
			}

			for node in &self.data.visual_nodes {
				frame.fill_rectangle(
					Point::new(node.bounds.x, node.bounds.y),
					Size::new(node.bounds.width, node.bounds.height),
					node.color,
				);

				let text_val = Text {
					content: node.label.clone(),
					position: Point::new(
						node.bounds.x + node.bounds.width + 5.0,
						node.bounds.y + (node.bounds.height / 4.0),
					),
					color: iced::Color::BLACK,
					size: iced::Pixels(12.0),
					..Default::default()
				};
				frame.fill_text(text_val);
			}
		});

		vec![geom]
	}
}
