//! This library implements a simple clustering algorithm with a partitive phase followed by an
//! agglomerative one.

/// Index pairs for similarity comparison.
pub mod index_pair;

/// Metrics for measuring similarity.
pub mod sim_metric;

/// Similarity matrix.
pub mod sim_matrix;

/// Clustering algorithm.
pub mod cluster;

/// Clustering evaluation metrics.
pub mod evaluation;

/// Index pairs for similarity comparison.
pub mod utils;


/// The `usize` count of elements in an input set.
pub type Size = usize;

/// A `usize` index into the input set to be clustered. Elements to be clustered are referred to by
/// their indices, rather than by their actual content.
pub type Index = usize;
