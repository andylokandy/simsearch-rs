use std::collections::HashMap;

use strsim::{levenshtein, normalized_levenshtein};

pub struct SearchOption {
    case_sensitive: bool,
    stop_whitespace: bool,
    stop_words: Option<&'static [&'static str]>,
    threadhold: f64,
}

impl SearchOption {
    pub fn new() -> Self {
        SearchOption {
            case_sensitive: false,
            stop_whitespace: true,
            stop_words: None,
            threadhold: 0.7,
        }
    }

    pub fn case_sensitive(self, case_sensitive: bool) -> Self {
        SearchOption {
            case_sensitive,
            ..self
        }
    }

    pub fn stop_whitespace(self, stop_whitespace: bool) -> Self {
        SearchOption {
            stop_whitespace,
            ..self
        }
    }

    pub fn stop_words(self, stop_words: &'static [&'static str]) -> Self {
        SearchOption {
            stop_words: Some(stop_words),
            ..self
        }
    }

    pub fn threadhold(self, threadhold: f64) -> Self {
        SearchOption { threadhold, ..self }
    }
}

pub struct SimSearch<Id>
where
    Id: PartialEq + Clone,
{
    option: SearchOption,
    id_next: usize,
    ids: Vec<(Id, usize)>,
    forward_map: HashMap<usize, Vec<String>>,
    reverse_map: HashMap<String, Vec<usize>>,
}

impl<Id> SimSearch<Id>
where
    Id: PartialEq + Clone,
{
    pub fn new() -> Self {
        Self::new_with(SearchOption::new())
    }

    pub fn new_with(option: SearchOption) -> Self {
        SimSearch {
            option,
            id_next: 0,
            ids: Vec::new(),
            forward_map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: Id, text: &str) {
        self.insert_tokens(id, &[text])
    }

    pub fn insert_tokens(&mut self, id: Id, tokens: &[&str]) {
        self.delete(&id);

        let id_num = self.id_next;
        self.id_next += 1;

        self.ids.push((id, id_num));

        let tokens = self.tokenize(tokens);

        for token in tokens.clone() {
            self.reverse_map
                .entry(token)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(id_num);
        }

        self.forward_map.insert(id_num, tokens);
    }

    pub fn delete(&mut self, id: &Id) {
        let id_num = self
            .ids
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, id_num)| id_num);
        if let Some(id_num) = id_num {
            for token in &self.forward_map[id_num] {
                self.reverse_map
                    .get_mut(token)
                    .unwrap()
                    .retain(|i| i != id_num);
            }
            self.forward_map.remove(id_num);
            self.ids.retain(|(i, _)| i != id);
        }
    }

    pub fn search(&self, pattern: &str) -> Vec<Id> {
        self.search_tokens(&[pattern])
    }

    pub fn search_tokens(&self, pattern_tokens: &[&str]) -> Vec<Id> {
        let pattern_tokens = self.tokenize(pattern_tokens);

        let mut token_scores: HashMap<&str, f64> = HashMap::new();

        for pattern_token in pattern_tokens {
            for token in self.reverse_map.keys() {
                let distance = levenshtein(&token, &pattern_token);
                let len_diff = token.len().saturating_sub(pattern_token.len());
                let score =
                    1. - ((distance.saturating_sub(len_diff)) as f64 / pattern_token.len() as f64);

                if score > self.option.threadhold {
                    let prefix_len = token.len() / 2;
                    let prefix_token =
                        String::from_utf8_lossy(token.as_bytes().split_at(prefix_len).0);
                    let score = (score
                        + normalized_levenshtein(&prefix_token, &pattern_token) as f64
                            / prefix_len as f64)
                        / 2.;
                    token_scores
                        .entry(token)
                        .and_modify(|current| *current = current.max(score))
                        .or_insert(0.);
                }
            }
        }

        let mut result_scores: HashMap<usize, f64> = HashMap::new();

        for (token, score) in token_scores.drain() {
            for id_num in &self.reverse_map[token] {
                *result_scores.entry(*id_num).or_insert(0.) += score;
            }
        }

        let mut result_scores: Vec<(usize, f64)> = result_scores.drain().collect();
        result_scores.sort_by(|lhs, rhs| rhs.1.partial_cmp(&lhs.1).unwrap());

        let result_ids: Vec<Id> = result_scores
            .iter()
            .map(|(id_num, _)| {
                self.ids
                    .iter()
                    .find(|(_, i)| i == id_num)
                    .map(|(id, _)| id.clone())
                    .unwrap()
            })
            .collect();

        result_ids
    }

    fn tokenize(&self, tokens: &[&str]) -> Vec<String> {
        let tokens: Vec<String> = tokens
            .iter()
            .map(|token| {
                if self.option.case_sensitive {
                    token.to_string()
                } else {
                    token.to_lowercase()
                }
            })
            .collect();

        let tokens: Vec<String> = if self.option.stop_whitespace {
            tokens
                .iter()
                .flat_map(|token| token.split_whitespace())
                .map(|token| token.to_string())
                .collect()
        } else {
            tokens
        };

        let mut tokens: Vec<String> = if let Some(stop_words) = self.option.stop_words {
            let mut tokens = tokens;
            for stop_word in stop_words {
                tokens = tokens
                    .iter()
                    .flat_map(|token| token.split_terminator(stop_word))
                    .map(|token| token.to_string())
                    .collect();
            }
            tokens
        } else {
            tokens
        };

        tokens.retain(|token| !token.is_empty());

        tokens
    }
}
