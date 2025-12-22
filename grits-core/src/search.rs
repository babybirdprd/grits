use crate::models::Issue;
use std::collections::{HashMap, HashSet};

pub struct SearchIndex {
    // Map word -> document frequency (count of issues containing this word)
    df: HashMap<String, usize>,
    // Map issue_id -> word counts
    tf: HashMap<String, HashMap<String, usize>>,
    // Map issue_id -> total word count (doc length)
    doc_lengths: HashMap<String, usize>,
    // Total number of documents
    total_docs: usize,
    // Average document length
    avg_dl: f64,
}

impl SearchIndex {
    pub fn new(issues: &[Issue]) -> Self {
        let mut index = SearchIndex {
            df: HashMap::new(),
            tf: HashMap::new(),
            doc_lengths: HashMap::new(),
            total_docs: issues.len(),
            avg_dl: 0.0,
        };

        let mut total_length = 0;

        for issue in issues {
            let text = format!("{} {}", issue.title, issue.description);
            let words = tokenize(&text);
            let doc_len = words.len();

            total_length += doc_len;
            index.doc_lengths.insert(issue.id.clone(), doc_len);

            let mut word_counts = HashMap::new();
            let mut unique_words = HashSet::new();

            for word in words {
                *word_counts.entry(word.clone()).or_insert(0) += 1;
                unique_words.insert(word);
            }

            index.tf.insert(issue.id.clone(), word_counts);

            for word in unique_words {
                *index.df.entry(word).or_insert(0) += 1;
            }
        }

        if index.total_docs > 0 {
            index.avg_dl = total_length as f64 / index.total_docs as f64;
        }

        index
    }

    pub fn search(&self, query: &str, issues: &[Issue]) -> Vec<(Issue, f64)> {
        let query_words = tokenize(query);
        let mut scores: HashMap<String, f64> = HashMap::new();

        let k1 = 1.2;
        let b = 0.75;

        for word in query_words {
            if let Some(doc_freq) = self.df.get(&word) {
                let idf = ((self.total_docs as f64 - *doc_freq as f64 + 0.5)
                    / (*doc_freq as f64 + 0.5)
                    + 1.0)
                    .ln();

                for (doc_id, term_freqs) in &self.tf {
                    if let Some(tf_val) = term_freqs.get(&word) {
                        let tf = *tf_val as f64;
                        let doc_len = *self.doc_lengths.get(doc_id).unwrap_or(&0) as f64;

                        let score = idf * (tf * (k1 + 1.0))
                            / (tf + k1 * (1.0 - b + b * (doc_len / self.avg_dl)));

                        *scores.entry(doc_id.clone()).or_insert(0.0) += score;
                    }
                }
            }
        }

        let mut results: Vec<(Issue, f64)> = issues
            .iter()
            .filter_map(|issue| {
                if let Some(score) = scores.get(&issue.id) {
                    Some((issue.clone(), *score))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}
