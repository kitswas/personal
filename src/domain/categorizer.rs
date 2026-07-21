use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Categorizer {
	regex_rules: Vec<(Regex, String)>,
	// simple Naive Bayes model based on word frequencies
	word_freqs: HashMap<String, HashMap<String, usize>>, // account_id -> word -> count
	account_priors: HashMap<String, usize>,              // account_id -> count
	total_transactions: usize,
}

impl Categorizer {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add_rule(
		&mut self,
		pattern: &str,
		account_id: &str,
	) -> Result<(), regex::Error> {
		// Use size_limit to prevent catastrophic backtracking as per Plan.md
		let re = regex::RegexBuilder::new(pattern)
			.size_limit(10 * 1024 * 1024)
			.build()?;
		self.regex_rules.push((re, account_id.to_string()));
		Ok(())
	}

	pub fn train(&mut self, payee: &str, account_id: &str) {
		let words = self.tokenize(payee);
		let entry = self
			.account_priors
			.entry(account_id.to_string())
			.or_insert(0);
		*entry += 1;
		self.total_transactions += 1;

		let word_map = self.word_freqs.entry(account_id.to_string()).or_default();
		for word in words {
			*word_map.entry(word).or_insert(0) += 1;
		}
	}

	fn tokenize(&self, text: &str) -> Vec<String> {
		text.to_lowercase()
			.split_whitespace()
			.map(|s| s.to_string())
			.collect()
	}

	pub fn categorize(&self, payee: &str) -> Option<(String, f32)> {
		// 1. Try Regex rules first
		for (re, account_id) in &self.regex_rules {
			if re.is_match(payee) {
				return Some((account_id.clone(), 1.0)); // 100% confidence
			}
		}

		// 2. Try Naive Bayes
		if self.total_transactions == 0 {
			return None;
		}

		let words = self.tokenize(payee);
		let mut best_score = f32::NEG_INFINITY;
		let mut best_account = None;

		let vocab_size = 1000.0; // Laplace smoothing vocabulary size heuristic

		for (account_id, prior_count) in &self.account_priors {
			let prior = (*prior_count as f32) / (self.total_transactions as f32);
			let mut log_prob = prior.ln();

			let total_words_in_class: usize = self
				.word_freqs
				.get(account_id)
				.map(|m| m.values().sum())
				.unwrap_or(0);

			for word in &words {
				let count = self
					.word_freqs
					.get(account_id)
					.and_then(|m| m.get(word))
					.copied()
					.unwrap_or(0);
				// Laplace smoothing
				let prob =
					(count as f32 + 1.0) / (total_words_in_class as f32 + vocab_size);
				log_prob += prob.ln();
			}

			if log_prob > best_score {
				best_score = log_prob;
				best_account = Some(account_id.clone());
			}
		}

		best_account.map(|acc| {
			// Pseudo-confidence score
			let confidence = (best_score.exp() * 100.0).clamp(0.0, 0.99);
			(acc, confidence)
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_regex_rules() {
		let mut categorizer = Categorizer::new();
		categorizer
			.add_rule(r"^UBER", "expenses:transport")
			.unwrap();

		let res = categorizer.categorize("UBER EATS");
		assert_eq!(res.unwrap().0, "expenses:transport");

		let res2 = categorizer.categorize("LYFT");
		assert!(res2.is_none()); // No rule, no ML data yet
	}

	#[test]
	fn test_naive_bayes() {
		let mut categorizer = Categorizer::new();
		categorizer.train("KROGER STORE 123", "expenses:groceries");
		categorizer.train("KROGER STORE 456", "expenses:groceries");
		categorizer.train("WHOLE FOODS", "expenses:groceries");
		categorizer.train("SHELL OIL", "expenses:gas");
		categorizer.train("CHEVRON", "expenses:gas");

		let res = categorizer.categorize("KROGER");
		assert_eq!(res.unwrap().0, "expenses:groceries");

		let res2 = categorizer.categorize("SHELL");
		assert_eq!(res2.unwrap().0, "expenses:gas");
	}
}
