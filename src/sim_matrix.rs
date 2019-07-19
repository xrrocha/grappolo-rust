//! This module contains the definition of a symmetric similarity matrix.

use std::collections::{HashMap, HashSet};
use std::ops::Index as BracketedIndex;

use itertools::sorted;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;

use crate::{Index, Size};
use crate::index_pair::IndexPair;
use crate::sim_metric::Similarity;
use std::cmp::Ordering;

/// Each cell in a row holds a sibling element's index and its similarity to the row's element.
#[derive(Debug)]
pub struct Score {
    pub sibling_index: Index,
    pub similarity: Similarity,
}

/// Each row contains similarities for qualifying siblings.
#[derive(Debug)]
pub struct Row {
    pub scores: Vec<Score>
}

/// A simple, sparse similarity matrix. While this matrix has as many rows as elements in the
/// input set, each row contains scores only for sibling elements whose similarity is above a
/// given `min_similarity`.
#[derive(Debug)]
pub struct SimilarityMatrix {
    /// The collection of rows, each holding zero or more scores consisting of the sibling index
    /// and its similarity to this row's element. Since this matrix is symmetric it holds that
    /// `matrix[row_index, column_index] = matrix[column_index, row_index]`.
    pub rows: Vec<Row>,

    /// The minimum similarity used for creating this matrix
    min_similarity: Similarity,

    /// The ordered set of distinct similarity values present in this matrix.
    pub similarity_values: Vec<Similarity>,
}

/// similarity matrix implementation.
impl SimilarityMatrix {
    /// Create a new instance of `SimilarityMatrix`.
    ///
    /// # Arguments
    ///
    /// * `elements` - The input set vector containing elements to be clustered.
    /// * `min_similarity` - The minimum score to consider two elements similar.
    /// * `index_pair_iterator` - The index pair iterator used to measure similarity  between to elements
    /// * `similarity_metric` - The similarity metric to apply for clustering.
    ///
    pub fn new<T, I, M>(
        elements: &Vec<T>,
        min_similarity: Similarity,
        index_pair_iterator: &mut I,
        similarity_metric: M,
    ) -> SimilarityMatrix
        where
            T: Sync + Send,
            I: Iterator<Item=IndexPair> + Send,
            M: Fn(&T, &T) -> Similarity + Sync,
    {
        let size = elements.len();
        assert!(size > 0, "Cannot create matrix from empty vector");

        let mut rows: Vec<Row> = Vec::with_capacity(size);
        for _ in 0..size {
            let row: Row = Row { scores: vec![] };
            rows.push(row);
        }

        let mut similarity_values = HashSet::new();

        let similarity_triplets =
            index_pair_iterator
                .par_bridge()
                .map(|(row, column)|
                    (row, column, similarity_metric(&elements[row], &elements[column])))
                .filter(|(_, _, similarity)| *similarity > 0.0 && *similarity >= min_similarity)
                .collect::<Vec<(Index, Index, Similarity)>>();

        for (row_index, column_index, similarity) in similarity_triplets {
            rows[row_index].scores.push(Score { sibling_index: column_index, similarity });
            rows[column_index].scores.push(Score { sibling_index: row_index, similarity });
            similarity_values.insert(similarity.to_string());
        }

        for i in 0..rows.len() {
            rows[i].scores.sort_by(
                |Score { sibling_index: _index_1, similarity: similarity_1 },
                 Score { sibling_index: _index_2, similarity: similarity_2 }|
                    similarity_2.partial_cmp(&similarity_1).unwrap());
        }

        let similarity_values = sorted(similarity_values)
            .map(|similarity| similarity.parse::<Similarity>().unwrap())
            .collect::<Vec<Similarity>>();

        SimilarityMatrix { rows, min_similarity, similarity_values }
    }

    /// Return the size of this matrix.
    pub fn size(&self) -> Size {
        self.rows.len()
    }

    /// Create a new similarity matrix that is a subset of this matrix.
    ///
    /// # Arguments
    ///
    /// * `indices` - Indices to extract from this matrix. Indices in this vector must be less than
    /// this matrix's `size`.
    /// * `min_similarity` The minimum similarity used to filter sibling elements in each row.
    ///
    /// # Return
    ///
    /// A new, boxed similarity matrix.
    pub fn spin_off(
        &self,
        indices: &Vec<Index>,
        min_similarity: Similarity)
        -> SimilarityMatrix
    {
        let map =
            indices
                .iter()
                .zip(0..indices.len())
                .map(|(new_index, old_index)| (*new_index, old_index))
                .collect::<HashMap<Index, Index>>();

        let index_set = indices.iter().map(|index| *index).collect::<HashSet<Index>>();

        let rows =
            indices
                .iter()
                .map(|previous_index| &self.rows[*previous_index])
                .map(|row|
                    Row {
                        scores: row.scores
                            .iter()
                            .filter(|score|
                                index_set.contains(&score.sibling_index) &&
                                    score.similarity >= min_similarity)
                            .map(|score|
                                Score {
                                    sibling_index: *map.get(&score.sibling_index).unwrap(),
                                    similarity: score.similarity,
                                })
                            .collect::<Vec<Score>>()
                    }
                )
                .collect::<Vec<Row>>();

        let similarity_values =
            sorted(
                rows
                    .iter()
                    .flat_map(|row|
                        row.scores
                            .iter()
                            .map(|Score { sibling_index: _, similarity }| similarity.to_string())
                            .collect::<Vec<String>>()
                    )
                    .collect::<HashSet<String>>()
            )
                .map(|string| string.parse::<Similarity>().unwrap())
                .collect::<Vec<Similarity>>();

        SimilarityMatrix { rows, min_similarity, similarity_values }
    }

    ///
    pub fn rank_by_weight(&self) -> Vec<Index> {
        let mut ordered_indices =
            (0..self.rows.len())
                .map(|index| (index, &self.rows[index]))
                .map(|(index, row)| {
                    let sibling_count = row.scores.len();
                    let similarity_sum =
                        row.scores.iter()
                            .map(|score| score.similarity)
                            .sum::<Similarity>();
                    (index, sibling_count, similarity_sum)
                })
                .collect::<Vec<(Index, Size, Similarity)>>();

        ordered_indices.sort_by(|(_, sibling_count1, similarity_sum_1), (_, sibling_count2, similarity_sum_2)| {
            if sibling_count1 > sibling_count2 {
                Ordering::Less
            } else if sibling_count1 == sibling_count2 && similarity_sum_1 > similarity_sum_2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        ordered_indices.iter()
            .map(|(index, _, _)| *index)
            .collect::<Vec<Index>>()
    }
}

/// Implementation of `std::ops::Index` for similarity matrix.
impl BracketedIndex<Index> for SimilarityMatrix {
    /// The data type of values returned by the indexing operator (`[]`).
    type Output = Row;

    /// Return the row at a given position position.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the desired row in this matrix.
    ///
    /// # Return
    ///
    /// The row at `index` position.
    fn index(&self, index: Size) -> &Self::Output {
        &self.rows[index]
    }
}

/// Implementation of `std::ops::Index` for `Score`.
impl BracketedIndex<Index> for Row {
    /// The data type of values returned by the indexing operator (`[]`).
    type Output = Similarity;

    /// Return the row at a given position position.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the desired row in this matrix.
    ///
    /// # Return
    ///
    /// The row at `index` position.
    fn index(&self, index: Size) -> &Self::Output {
        &self.scores
            .iter()
            .find(|score| score.sibling_index == index)
            .map(|score| &score.similarity)
            .unwrap_or(&0.0)
    }
}

impl Row {
    pub fn new(scores: Vec<Score>) -> Row {
        Row { scores }
    }

    pub fn cut_at(&self, similarity: Similarity) -> Vec<Index> {
        self.scores.iter()
            .filter(|score| score.similarity >= similarity)
            .map(|score| score.sibling_index)
            .collect::<Vec<Index>>()
    }

    pub fn ranked_siblings(&self, excluding: &HashSet<Index>) -> Vec<Index> {
        let mut siblings =
            self.scores.iter()
                .filter(|score| !excluding.contains(&score.sibling_index))
                .collect::<Vec<&Score>>();

        siblings.sort_by(|score_1, score_2|
            (*score_2).similarity.partial_cmp(&score_1.similarity).unwrap());

        siblings.iter()
            .map(|score| score.sibling_index)
            .collect::<Vec<Index>>()
    }
}


#[cfg(test)]
mod tests {
    use strsim::normalized_damerau_levenshtein;

    use crate::index_pair::cartesian::CartesianIndexPairIterator;
    use crate::utils::string_vec;

    use super::*;

    type Scores = (Index, Index, Similarity);

    #[test]
    #[should_panic]
    fn new_sparse_similarity_matrix_rejects_zero_size() {
        let empty_vec = vec![];

        SimilarityMatrix::new(
            &empty_vec,
            0.6,
            &mut CartesianIndexPairIterator::new(2),
            |t1: &String, t2: &String| normalized_damerau_levenshtein(t1.as_str(), t2.as_str()),
        );
    }

    #[test]
    fn matrix_holds_correct_scores() {
        let (names, scores) = name_scores();

        let similarity_matrix = SimilarityMatrix::new(
            &names,
            0.0,
            &mut CartesianIndexPairIterator::new(names.len()),
            |t1: &String, t2: &String| normalized_damerau_levenshtein(t1.as_str(), t2.as_str()),
        );

        check_scores(&similarity_matrix, names.len(), scores);
    }

    #[test]
    fn matrix_creates_proper_spin_off() {
        let (names, _) = name_scores();

        let partial_min_similarity = 0.4;

        let partial_indices: Vec<Index> = vec![
            2, // 0: martha
            3, // 1: marta
            4, // 2: marlene
            5, // 3: marleny
            6, // 4: malrene
        ]
            .iter()
            .map(|index| *index)
            .collect::<Vec<Index>>();
        let size = (&partial_indices).len();

        let partial_scores: Vec<(Index, Index, Similarity)> = vec![
            (0, 1, 0.8333333333333334), // (martha, marta)
            (0, 2, 0.4285714285714286), // (martha, marlene)
            (0, 3, 0.4285714285714286), // (martha, marleny)
            (0, 4, 0.4285714285714286), // (martha, malrene)
            (1, 2, 0.4285714285714286), // (marta, marlene)
            (1, 3, 0.4285714285714286), // (marta, marleny)
            (1, 4, 0.4285714285714286), // (marta, malrene)
            (2, 3, 0.8571428571428572), // (marlene, marleny)
            (2, 4, 0.8571428571428572), // (marlene, malrene)
            (3, 4, 0.7142857142857143), // (marleny, malrene)
        ];

        let similarity_matrix = SimilarityMatrix::new(
            &names,
            0.0,
            &mut CartesianIndexPairIterator::new(names.len()),
            |t1: &String, t2: &String| normalized_damerau_levenshtein(t1.as_str(), t2.as_str()),
        );

        let similarity_matrix =
            similarity_matrix.spin_off(&partial_indices, partial_min_similarity);

        check_scores(&similarity_matrix, size, partial_scores)
    }

    fn check_scores(similarity_matrix: &SimilarityMatrix, size: Size, scores: Vec<Scores>) {
        assert_eq!(similarity_matrix.size(), size);

        for (row, column, expected_similarity) in scores {
            assert_eq!(similarity_matrix[row][column], expected_similarity);
            assert_eq!(similarity_matrix[column][row], expected_similarity);
        }
    }

    fn name_scores() -> (Vec<String>, Vec<(Index, Index, Similarity)>) {
        (
            string_vec(vec![
                "alejandro", "alejo",
                "martha", "marta",
                "marlene", "marleny", "malrene",
                "ricardo"
            ]),
            vec![
                (0, 1, 0.5555555555555556), // (alejandro, alejo)
                (0, 2, 0.11111111111111116), // (alejandro, martha)
                (0, 3, 0.11111111111111116), // (alejandro, marta)
                (0, 4, 0.2222222222222222), // (alejandro, marlene)
                (0, 5, 0.2222222222222222), // (alejandro, marleny)
                (0, 6, 0.2222222222222222), // (alejandro, malrene)
                (0, 7, 0.33333333333333337), // (alejandro, ricardo)
                (1, 2, 0.16666666666666663), // (alejo, martha)
                (1, 3, 0.0), // (alejo, marta)
                (1, 4, 0.4285714285714286), // (alejo, marlene)
                (1, 5, 0.4285714285714286), // (alejo, marleny)
                (1, 6, 0.4285714285714286), // (alejo, malrene)
                (1, 7, 0.1428571428571429), // (alejo, ricardo)
                (2, 3, 0.8333333333333334), // (martha, marta)
                (2, 4, 0.4285714285714286), // (martha, marlene)
                (2, 5, 0.4285714285714286), // (martha, marleny)
                (2, 6, 0.4285714285714286), // (martha, malrene)
                (2, 7, 0.1428571428571429), // (martha, ricardo)
                (3, 4, 0.4285714285714286), // (marta, marlene)
                (3, 5, 0.4285714285714286), // (marta, marleny)
                (3, 6, 0.4285714285714286), // (marta, malrene)
                (3, 7, 0.2857142857142857), // (marta, ricardo)
                (4, 5, 0.8571428571428572), // (marlene, marleny)
                (4, 6, 0.8571428571428572), // (marlene, malrene)
                (4, 7, 0.0), // (marlene, ricardo)
                (5, 6, 0.7142857142857143), // (marleny, malrene)
                (5, 7, 0.0), // (marleny, ricardo)
                (6, 7, 0.0), // (malrene, ricardo)
            ])
    }
}



