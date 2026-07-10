use std::collections::{HashMap, HashSet};

/// Simple Naive Bayes classifier for mapping payee strings to account IDs.
#[derive(Default)]
pub struct Classifier {
	// account_id -> (word -> count)
	class_word_counts: HashMap<String, HashMap<String, u32>>,
	// account_id -> total words in this class
	class_total_words: HashMap<String, u32>,
	// total documents (transactions) per class
	class_docs: HashMap<String, u32>,
	// total documents overall
	total_docs: u32,
	// vocabulary size
	vocab_size: u32,
}

impl Classifier {
	pub fn new() -> Self {
		Self {
			class_word_counts: HashMap::new(),
			class_total_words: HashMap::new(),
			class_docs: HashMap::new(),
			total_docs: 0,
			vocab_size: 0,
		}
	}

	/// Train the model on historical transactions from the database.
	pub fn train(&mut self, history: impl Iterator<Item = (String, String)>) {
		let mut vocab = HashSet::new();

		for (payee, account_id) in history {
			self.total_docs += 1;
			*self.class_docs.entry(account_id.clone()).or_insert(0) += 1;

			let words = tokenize(&payee);
			for w in words {
				vocab.insert(w.clone());
				let class_words = self
					.class_word_counts
					.entry(account_id.clone())
					.or_default();
				*class_words.entry(w).or_insert(0) += 1;
				*self
					.class_total_words
					.entry(account_id.clone())
					.or_insert(0) += 1;
			}
		}

		self.vocab_size = vocab.len() as u32;
	}

	/// Predict the best account ID and return it with a confidence score.
	pub fn predict(&self, payee: &str) -> Option<(String, f32)> {
		if self.total_docs == 0 {
			return None;
		}

		let words = tokenize(payee);
		let mut max_log_prob = f64::NEG_INFINITY;
		let mut best_class = None;

		for (class, docs_in_class) in &self.class_docs {
			let prior_prob = (*docs_in_class as f64) / (self.total_docs as f64);
			let mut log_prob = prior_prob.ln();

			let total_words_in_class =
				self.class_total_words.get(class).copied().unwrap_or(0);
			let denom = (total_words_in_class + self.vocab_size) as f64;

			for w in &words {
				let count = self
					.class_word_counts
					.get(class)
					.and_then(|cw| cw.get(w))
					.copied()
					.unwrap_or(0);

				// Laplace smoothing
				let word_prob = (count as f64 + 1.0) / denom;
				log_prob += word_prob.ln();
			}

			if log_prob > max_log_prob {
				max_log_prob = log_prob;
				best_class = Some(class.clone());
			}
		}

		// Extremely naive confidence approximation
		let confidence = if max_log_prob > -5.0 { 0.9 } else { 0.4 };

		best_class.map(|c| (c, confidence))
	}
}

fn tokenize(text: &str) -> Vec<String> {
	text.to_lowercase()
		.split_whitespace()
		.map(|s| {
			s.chars()
				.filter(|c| c.is_alphanumeric())
				.collect::<String>()
		})
		.filter(|s| !s.is_empty())
		.collect()
}
