//! A simple and lightweight fuzzy search engine that works in memory, searching for
//! similar strings (a pun here).
//!
//! # Examples
//!
//! ```
//! use simsearch::SimSearch;
//!
//! let mut engine: SimSearch<u32> = SimSearch::new();
//!
//! engine.insert(1, "Things Fall Apart");
//! engine.insert(2, "The Old Man and the Sea");
//! engine.insert(3, "James Joyce");
//!
//! let results: Vec<u32> = engine.search("thngs");
//!
//! assert_eq!(results, &[1]);
//! ```

use std::collections::HashMap;

use strsim::{levenshtein, normalized_levenshtein};

/// The simple search engine.
pub struct SimSearch<Id>
where
    Id: PartialEq + Clone,
{
    option: SearchOptions,
    id_next: usize,
    ids: Vec<(Id, usize)>,
    forward_map: HashMap<usize, Vec<String>>,
    reverse_map: HashMap<String, Vec<usize>>,
}

impl<Id> SimSearch<Id>
where
    Id: PartialEq + Clone,
{
    /// Creates search engine with default options.
    pub fn new() -> Self {
        Self::new_with(SearchOptions::new())
    }

    /// Creates search engine with custom options.
    ///
    /// # Examples
    ///
    /// ```
    /// use simsearch::{SearchOptions, SimSearch};
    ///
    /// let mut engine: SimSearch<usize> = SimSearch::new_with(
    ///     SearchOptions::new().case_sensitive(true));
    /// ```
    pub fn new_with(option: SearchOptions) -> Self {
        SimSearch {
            option,
            id_next: 0,
            ids: Vec::new(),
            forward_map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    /// Inserts an entry into search engine.
    ///
    /// Input will be tokenized by the built-in tokenizer,
    /// by default whitespaces(including tabs) are considered as stop words,
    /// you can change the behavior by providing `SearchOptions`.
    ///
    /// Search engine will delete the existing entry
    /// with same id before inserting the new one.
    ///
    /// **Note that** id is not searchable. Add id to the contents if you would
    /// like to perform search on it.
    ///
    /// # Examples
    ///
    /// ```
    /// use simsearch::{SearchOptions, SimSearch};
    ///
    /// let mut engine: SimSearch<&str> = SimSearch::new_with(
    ///     SearchOptions::new().stop_words(&[",", "."]));
    ///
    /// engine.insert("BoJack Horseman", "BoJack Horseman, an American
    /// adult animated comedy-drama series created by Raphael Bob-Waksberg.
    /// The series stars Will Arnett as the title character,
    /// with a supporting cast including Amy Sedaris,
    /// Alison Brie, Paul F. Tompkins, and Aaron Paul.");
    /// ```
    pub fn insert(&mut self, id: Id, content: &str) {
        self.insert_tokenized(id, &[content])
    }

    /// Inserts a pre-tokenized entry into search engine.
    ///
    /// Search engine will apply built-in tokenizer on the
    /// provided tokens again. Use this method when you have
    /// special tokenizing rules in addition to the built-in ones.
    ///
    /// Search engine will delete the existing entry
    /// with same id before inserting the new one.
    ///
    /// **Note that** id is not searchable. Add id to the contents if you would
    /// like to perform search on it.
    ///
    /// # Examples
    ///
    /// ```
    /// use simsearch::SimSearch;
    ///
    /// let mut engine: SimSearch<&str> = SimSearch::new();
    ///
    /// engine.insert_tokenized("Arya Stark", &["Arya Stark", "a fictional
    /// character in American author George R. R", "portrayed by English actress."]);
    pub fn insert_tokenized(&mut self, id: Id, tokens: &[&str]) {
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

    /// Searches for pattern and returns ids sorted by relevance.
    ///
    /// Pattern will be tokenized by the built-in tokenizer,
    /// by default whitespaces(including tabs) are considered as stop words,
    /// you can change the behavior by providing `SearchOptions`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simsearch::SimSearch;
    ///
    /// let mut engine: SimSearch<u32> = SimSearch::new();
    ///
    /// engine.insert(1, "Things Fall Apart");
    /// engine.insert(2, "The Old Man and the Sea");
    /// engine.insert(3, "James Joyce");
    ///
    /// let results: Vec<u32> = engine.search("thngs apa");
    ///
    /// assert_eq!(results, &[1]);
    pub fn search(&self, pattern: &str) -> Vec<Id> {
        self.search_tokenized(&[pattern])
    }

    /// Searches for pre-tokenized pattern and returns ids sorted by relevance.
    ///
    /// Search engine will apply built-in tokenizer on the provided
    /// tokens again. Use this method when you have special
    /// tokenizing rules in addition to the built-in ones.
    ///
    /// # Examples
    ///
    /// ```
    /// use simsearch::SimSearch;
    ///
    /// let mut engine: SimSearch<u32> = SimSearch::new();
    ///
    /// engine.insert(1, "Things Fall Apart");
    /// engine.insert(2, "The Old Man and the Sea");
    /// engine.insert(3, "James Joyce");
    ///
    /// let results: Vec<u32> = engine.search_tokenized(&["thngs", "apa"]);
    ///
    /// assert_eq!(results, &[1]);
    pub fn search_tokenized(&self, pattern_tokens: &[&str]) -> Vec<Id> {
        let pattern_tokens = self.tokenize(pattern_tokens);

        let mut token_scores: HashMap<&str, f64> = HashMap::new();

        for pattern_token in pattern_tokens {
            for token in self.reverse_map.keys() {
                let distance = levenshtein(&token, &pattern_token);
                let len_diff = token.len().saturating_sub(pattern_token.len());
                let score =
                    1. - ((distance.saturating_sub(len_diff)) as f64 / pattern_token.len() as f64);

                if score > self.option.threshold {
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
            }).collect();

        result_ids
    }

    /// Deletes entry by id.
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

    fn tokenize(&self, tokens: &[&str]) -> Vec<String> {
        let tokens: Vec<String> = tokens
            .iter()
            .map(|token| {
                if self.option.case_sensitive {
                    token.to_string()
                } else {
                    token.to_lowercase()
                }
            }).collect();

        let mut tokens: Vec<String> = if self.option.stop_whitespace {
            tokens
                .iter()
                .flat_map(|token| token.split_whitespace())
                .map(|token| token.to_string())
                .collect()
        } else {
            tokens
        };

        for stop_word in self.option.stop_words {
            tokens = tokens
                .iter()
                .flat_map(|token| token.split_terminator(stop_word))
                .map(|token| token.to_string())
                .collect();
        }

        tokens.retain(|token| !token.is_empty());

        tokens
    }
}

/// Options and flags which can be used to configure how the search engine works.
///
/// # Examples
///
/// ```
/// use simsearch::{SearchOptions, SimSearch};
///
/// let mut engine: SimSearch<usize> = SimSearch::new_with(
///     SearchOptions::new().case_sensitive(true));
/// ```
pub struct SearchOptions {
    case_sensitive: bool,
    stop_whitespace: bool,
    stop_words: &'static [&'static str],
    threshold: f64,
}

impl SearchOptions {
    /// Creates a blank new set of options ready for configuration.
    pub fn new() -> Self {
        SearchOptions {
            case_sensitive: false,
            stop_whitespace: true,
            stop_words: &[],
            threshold: 0.7,
        }
    }

    /// Sets the option for case sensitive.
    ///
    /// Defaults to `false`.
    pub fn case_sensitive(self, case_sensitive: bool) -> Self {
        SearchOptions {
            case_sensitive,
            ..self
        }
    }

    /// Sets the option for whitespace tokenizing.
    ///
    /// This option enables built-in tokenizer to split entry contents
    /// or search patterns by UTF-8 whitespace (including tab, returns
    /// and so forth).
    ///
    /// See also [`std::str::split_whitespace()`](https://doc.rust-lang.org/std/primitive.str.html#method.split_whitespace).
    ///
    /// Defaults to `true`.
    pub fn stop_whitespace(self, stop_whitespace: bool) -> Self {
        SearchOptions {
            stop_whitespace,
            ..self
        }
    }

    /// Sets the option for custom tokenizing.
    ///
    /// This option enables built-in tokenizer to split entry contents
    /// or search patterns by a list of custom stop words.
    ///
    /// Defaults to be an empty list `&[]`.
    ///
    /// # Examples
    /// ```
    /// use simsearch::{SearchOptions, SimSearch};
    ///
    /// let mut engine: SimSearch<usize> = SimSearch::new_with(
    ///     SearchOptions::new().stop_words(&["/", "\\"]));
    ///
    /// engine.insert(1, "the old/man/and/the sea");
    ///
    /// let results = engine.search("old");
    ///
    /// assert_eq!(results, &[1]);
    /// ```
    pub fn stop_words(self, stop_words: &'static [&'static str]) -> Self {
        SearchOptions { stop_words, ..self }
    }

    /// Sets the threshold for search scoring.
    ///
    /// Search results will be sorted by their scores. Scores
    /// ranges from 0 to 1 when the 1 indicates the most relevant.
    /// Only the entries with scores greater than threshold will be returned.
    ///
    /// Defaults to `0.7`.
    pub fn threshold(self, threshold: f64) -> Self {
        SearchOptions { threshold, ..self }
    }
}
