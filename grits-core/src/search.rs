use crate::models::Issue;
use std::collections::{HashMap, HashSet};

const K1: f64 = 1.2;
const B: f64 = 0.75;

#[derive(Default)]
pub struct SearchIndex {
    // Term -> Document ID -> Term Frequency
    inverted_index: HashMap<String, HashMap<String, usize>>,
    // Document ID -> Field Lengths (title + description)
    doc_lengths: HashMap<String, usize>,
    // Average Document Length
    avg_doc_length: f64,
    // Total documents
    total_docs: usize,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index_issues(&mut self, issues: &[Issue]) {
        self.inverted_index.clear();
        self.doc_lengths.clear();
        self.total_docs = issues.len();

        let mut total_length = 0;

        for issue in issues {
            let text = format!("{} {}", issue.title, issue.description);
            let tokens = self.tokenize(&text);
            let doc_len = tokens.len();

            self.doc_lengths.insert(issue.id.clone(), doc_len);
            total_length += doc_len;

            let mut term_freqs = HashMap::new();
            for token in tokens {
                *term_freqs.entry(token).or_insert(0) += 1;
            }

            for (term, freq) in term_freqs {
                self.inverted_index
                    .entry(term)
                    .or_default()
                    .insert(issue.id.clone(), freq);
            }
        }

        self.avg_doc_length = if self.total_docs > 0 {
            total_length as f64 / self.total_docs as f64
        } else {
            0.0
        };
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<(String, f64)> {
        let query_tokens = self.tokenize(query);
        let mut scores: HashMap<String, f64> = HashMap::new();

        for token in query_tokens {
            if let Some(doc_freqs) = self.inverted_index.get(&token) {
                let doc_count_with_term = doc_freqs.len();
                let idf = self.calculate_idf(doc_count_with_term);

                for (doc_id, freq) in doc_freqs {
                    let doc_len = *self.doc_lengths.get(doc_id).unwrap_or(&0) as f64;
                    let tf = *freq as f64;

                    let score = idf * (tf * (K1 + 1.0)) / (tf + K1 * (1.0 - B + B * (doc_len / self.avg_doc_length)));
                    *scores.entry(doc_id.clone()).or_insert(0.0) += score;
                }
            }
        }

        let mut result: Vec<_> = scores.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    fn calculate_idf(&self, doc_count_with_term: usize) -> f64 {
        let n = self.total_docs as f64;
        let n_q = doc_count_with_term as f64;
        ((n - n_q + 0.5) / (n_q + 0.5) + 1.0).ln()
    }
}
