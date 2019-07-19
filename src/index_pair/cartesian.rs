use crate::{Index, Size};

use super::IndexPair;

/// Cartesian product strategy for index pair iterator.
#[derive(Debug)]
pub struct CartesianIndexPairIterator {
    /// The number of elements in the input set.
    size: Size,
    /// Holder for the current iteration row .
    row: Index,
    /// Holder for the current iteration column.
    column: Index,
}

/// Implementation of `CartesianIndexPairIterator`.
impl CartesianIndexPairIterator {
    /// Return a new cartesian index pair iterator for a given set size.
    ///
    /// # Arguments
    /// * `size` - The element count for the input set.
    /// * `row` - The current iteration row.
    /// * `column` - The current iteration column.
    pub fn new(size: Size) -> CartesianIndexPairIterator {
        assert!(size > 0, "Size must be positive");

        CartesianIndexPairIterator {
            size,
            row: 0,
            column: 0,
        }
    }
}

/// Implement `Iterator<Item = PairIndex>` for `CartesianIndexPairIterator`
impl Iterator for CartesianIndexPairIterator {
    /// The iterator's `Item` type.
    type Item = IndexPair;

    /// Return the next index pair.
    fn next(&mut self) -> Option<IndexPair> {
        if self.row == self.size || self.size == 1 {
            None
        } else {
            self.column += 1;
            if self.column == self.size {
                self.row += 1;
                if self.row == self.size - 1 {
                    None
                } else {
                    self.column = self.row + 1;
                    Some((self.row, self.column))
                }
            } else {
                Some((self.row, self.column))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn new_cartesian_index_pairator_rejects_zero_size() {
        CartesianIndexPairIterator::new(0);
    }

    #[test]
    fn new_cartesian_index_pairator_sets_size() {
        assert_eq!(42, CartesianIndexPairIterator::new(42).size);
    }

    #[test]
    fn new_cartesian_index_pairator_sets_row_to_zero() {
        assert_eq!(CartesianIndexPairIterator::new(42).row, 0);
    }

    #[test]
    fn new_cartesian_index_pairator_sets_column_to_zero() {
        assert_eq!(CartesianIndexPairIterator::new(42).row, 0);
    }

    #[test]
    fn cartesian_index_pairator_yields_empty_set_for_singleton() {
        assert_eq!(CartesianIndexPairIterator::new(1).next(), None);
    }

    #[test]
    fn cartesian_index_pairator_yields_correct_pairs() {
        let expected_pairs: Vec<IndexPair> = vec![
            (0, 1),
            (0, 2),
            (1, 2),
        ];
        let expected_pair_count = &expected_pairs.len();

        let actual_pairs =
            CartesianIndexPairIterator::new(3)
                .collect::<Vec<IndexPair>>();
        let actual_pair_count = &actual_pairs.len();

        assert!(
            expected_pair_count == actual_pair_count,
            "Cartesian pair count mismatch. Expected {}, got {}",
            expected_pair_count, actual_pair_count
        );

        let same_values =
            &expected_pairs
                .iter()
                .zip(&actual_pairs)
                .all(|((expected_1, expected_2), (actual_1, actual_2))|
                    expected_1 == actual_1 && expected_2 == actual_2
                );

        assert!(
            same_values,
            "Cartesian pair result mismatch. Expected {:?}, got {:?}",
            expected_pairs, actual_pairs
        );
    }
}
