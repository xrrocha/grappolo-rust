//! This module contains an implementation of index pair iterator for strings. String pairs are
//! selected based on sharing one or more n-grams of a given length.
use std::collections::{HashMap, HashSet};

use crate::{Index, Size};

use super::IndexPair;

/// The NGram pair iterator structure
#[derive(Debug)]
pub struct NGramPairs {
    /// The collected index pairs.
    pairs: Vec<IndexPair>,
    /// The current iteration index.
    current_index: Index,
}

/// NGram implementation.
impl NGramPairs {
    /// Create a new `NGramPairs` instance
    ///
    /// # Arguments
    ///
    /// * `strings` - Reference to a vector of strings.
    /// * `ngram_length` - The length of n-grams to build in ascertaining commonality.
    ///
    /// # Return
    ///
    /// * A new `NGramPairs` instance.
    pub fn new(strings: &Vec<String>, ngram_length: Size) -> NGramPairs {
        assert!(ngram_length > 0);

        let size = strings.len();

        let mut ngram_to_indices = HashMap::new();
        (0..size)
            .flat_map(|index| {
                ngrams(&strings[index], ngram_length)
                    .iter()
                    .map(|ngram| (ngram.clone(), index))
                    .collect::<Vec<(String, Index)>>()
            })
            .for_each(|(ngram, index)| {
                ngram_to_indices
                    .entry(ngram)
                    .or_insert_with(|| HashSet::new())
                    .insert(index);
            });

        let mut index_to_ngrams: HashMap<Index, HashSet<Index>> = HashMap::new();
        ngram_to_indices.iter().for_each(|(ngram, indices)| {
            indices.iter().for_each(|index| {
                ngram_to_indices
                    .get(ngram)
                    .unwrap()
                    .iter()
                    .filter(|sibling_index| **sibling_index > *index)
                    .for_each(|sibling_index| {
                        index_to_ngrams
                            .entry(*index)
                            .or_insert_with(|| HashSet::new())
                            .insert(*sibling_index);
                    });
            });
        });

        let pairs: Vec<IndexPair> = index_to_ngrams
            .iter()
            .flat_map(|(index, sibling_indices)| {
                sibling_indices
                    .iter()
                    .map(|sibling_index| (*index, *sibling_index))
                    .collect::<Vec<IndexPair>>()
            })
            .collect::<Vec<IndexPair>>();

        NGramPairs { pairs, current_index: 0 }
    }
}

/// `MGramPairs` implementation of `Iterator<Item = IndexPair>`.
impl Iterator for NGramPairs {
    type Item = IndexPair;

    /// Iterator implementation function.
    fn next(&mut self) -> Option<IndexPair> {
        if self.current_index == self.pairs.len() {
            None
        } else {
            let pair = self.pairs[self.current_index];
            self.current_index += 1;
            Some(pair)
        }
    }
}


/// Divide a string into a set of (possibly duplicate) ngrams of a given length.
///
/// # Arguments
///
/// * `string` - Reference to string from which to extract n-grams.
///
/// # Return
///
/// A vector of strings containing all n-grams of the given length.
fn ngrams(string: &String, ngram_length: Size) -> Vec<String> {
    let last = string.len() - ngram_length + 1;
    (0..last)
        .map(|start| {
            let end = start + ngram_length;
            string.chars().take(end).skip(start).collect::<String>()
        })
        .filter(|ngram| ngram.len() == ngram_length)
        .collect::<Vec<String>>()
}

#[cfg(test)]
mod tests {
    use crate::utils::string_vec;

    use super::*;

    #[test]
    fn builds_ngrams_correctly() {
        let string = String::from("rustinomicon");

        let ngrams_2 = ngrams(&string, 2);
        assert_eq!(ngrams_2, string_vec(vec![
            "ru", "us", "st", "ti", "in", "no", "om", "mi", "ic", "co", "on",
        ]));

        let ngrams_3 = ngrams(&string, 3);
        assert_eq!(ngrams_3, string_vec(vec![
            "rus", "ust", "sti", "tin", "ino", "nom", "omi", "mic", "ico", "con",
        ]));
    }

    #[test]
    fn builds_pairs_correctly() {
        let names = string_vec(vec!["alejandro", "marlene", "martha", "ricardo"]);
        let expected_pairs = vec![
            (0usize, 1usize),
            (1, 2),
            (1, 3),
            (2, 3),
        ].iter().map(|p| *p).collect::<HashSet<IndexPair>>();

        let actual_pairs =
            NGramPairs::new(&names, 2)
                .collect::<HashSet<IndexPair>>();

        assert_eq!(actual_pairs, expected_pairs);
    }
}

