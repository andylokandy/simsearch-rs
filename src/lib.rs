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

use strsim::jaro_winkler;

/// The simple search engine.
pub struct SimSearch<Id>
where
    Id: PartialEq + Clone,
{
    option: SearchOptions,
    ids: Vec<Option<Id>>,
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
            ids: Vec::new(),
            forward_map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    /// Inserts an entry into search engine.
    ///
    /// Input will be tokenized according to the search option.
    /// By default whitespaces(including tabs) are considered as stop words,
    /// you can change the behavior by providing `SearchOptions`.
    ///
    /// Insert with an existing id updates the content.
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
        self.insert_tokens(id, &[content])
    }

    /// Inserts entry tokens into search engine.
    ///
    /// Search engine also applies tokenizer to the
    /// provided tokens. Use this method when you have
    /// special tokenization rules in addition to the built-in ones.
    ///
    /// Insert with an existing id updates the content.
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
    /// engine.insert_tokens("Arya Stark", &["Arya Stark", "a fictional
    /// character in American author George R. R", "portrayed by English actress."]);
    pub fn insert_tokens(&mut self, id: Id, tokens: &[&str]) {
        self.delete(&id);

        let id_num = self.ids.len();
        self.ids.push(Some(id));

        let mut tokens = self.tokenize(tokens);
        tokens.sort();
        tokens.dedup();

        for token in tokens.clone() {
            self.reverse_map
                .entry(token)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(id_num);
        }

        self.forward_map.insert(id_num, tokens);
    }

    /// Searches pattern and returns ids sorted by relevance.
    ///
    /// Pattern will be tokenized according to the search option.
    /// By default whitespaces(including tabs) are considered as stop words,
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
        self.search_tokens(&[pattern])
    }

    /// Searches pattern tokens and returns ids sorted by relevance.
    ///
    /// Search engine also applies tokenizer to the
    /// provided tokens. Use this method when you have
    /// special tokenization rules in addition to the built-in ones.
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
    /// let results: Vec<u32> = engine.search_tokens(&["thngs", "apa"]);
    ///
    /// assert_eq!(results, &[1]);
    pub fn search_tokens(&self, pattern_tokens: &[&str]) -> Vec<Id> {
        let mut pattern_tokens = self.tokenize(pattern_tokens);
        pattern_tokens.sort();
        pattern_tokens.dedup();

        let mut token_scores: HashMap<&str, f64> = HashMap::new();

        for pattern_token in pattern_tokens {
            for token in self.reverse_map.keys() {
                let score = jaro_winkler(token, &pattern_token);
                if score > self.option.threshold {
                    token_scores.insert(token, score);
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
                self.ids[*id_num]
                    .as_ref()
                    .map(|id| id.clone())
                    // this can go wrong only if something (e.g. delete) leaves us in an
                    // inconsistent state
                    .expect("id at id_num should be there")
            }).collect();

        result_ids
    }

    /// Deletes entry by id.
    pub fn delete(&mut self, id: &Id) {
        let id_num = self.ids.iter().position(|i| i.as_ref() == Some(id));
        if let Some(id_num) = id_num {
            for token in &self.forward_map[&id_num] {
                self.reverse_map
                    .get_mut(token)
                    .unwrap()
                    .retain(|i| *i != id_num);
            }
            self.forward_map.remove(&id_num);
            self.ids[id_num] = None;
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

/// Options and flags that configuring the search engine.
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
    /// Creates a default configuration.
    pub fn new() -> Self {
        SearchOptions {
            case_sensitive: false,
            stop_whitespace: true,
            stop_words: &[],
            threshold: 0.8,
        }
    }

    /// Sets whether search engine is case sensitive or not.
    ///
    /// Defaults to `false`.
    pub fn case_sensitive(self, case_sensitive: bool) -> Self {
        SearchOptions {
            case_sensitive,
            ..self
        }
    }

    /// Sets the whether search engine splits tokens on whitespace or not.
    /// The **whitespace** here includes tab, returns and so forth.
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

    /// Sets the custom token stop word.
    ///
    /// This option enables tokenizer to split contents
    /// and search words by the extra list of custom stop words.
    ///
    /// Defaults to `&[]`.
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
    /// Search results will be sorted by their Jaro winkler similarity scores.
    /// Scores ranges from 0 to 1 where the 1 indicates the most relevant.
    /// Only the entries with scores greater than the threshold will be returned.
    ///
    /// Defaults to `0.8`.
    pub fn threshold(self, threshold: f64) -> Self {
        SearchOptions { threshold, ..self }
    }
}
