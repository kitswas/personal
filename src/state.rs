use std::collections::HashSet;

pub struct AppState {
	pub total_balance: f64,
	pub expanded_nodes: HashSet<String>,
}

impl AppState {
	pub fn new() -> Self {
		Self {
			total_balance: 0.0,
			expanded_nodes: HashSet::new(),
		}
	}

	pub fn apply_message(&mut self, msg: Message) -> Command {
		match msg {
			Message::ToggleSankeyNode(node_id) => {
				if self.expanded_nodes.contains(&node_id) {
					self.expanded_nodes.remove(&node_id);
				} else {
					self.expanded_nodes.insert(node_id);
				}
				Command::None
			},
			Message::FetchData => Command::LoadData,
			Message::DataLoaded(Ok(balance)) => {
				self.total_balance = balance;
				Command::None
			},
			Message::DataLoaded(Err(e)) => {
				eprintln!("Error loading data: {}", e);
				Command::None
			},
		}
	}
}

pub enum Message {
	ToggleSankeyNode(String),
	FetchData,
	DataLoaded(Result<f64, String>),
}

pub enum Command {
	None,
	LoadData,
}
