//! This module defines index pair generation strategies.

use crate::Index;

pub mod ngrams;
pub mod cartesian;

/// Pair of indices corresponding to candidate elements to be considered for clustering together.
pub type IndexPair = (Index, Index);

/// Iterator over index pairs to be considered for clustering together.
pub type IndexPairIterator = Iterator<Item=IndexPair> + Send;
