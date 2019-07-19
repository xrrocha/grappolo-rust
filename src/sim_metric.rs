//! This module defines specifies how similarity between two items is established.

/// Similarity is a normalized value between `0.0` (no similarity at all) and `1.0` (actual
/// identity). Similarity is the opposite of *distance*.
pub type Similarity = f64;

/// Measure the similarity between two values of a given type.
pub type SimilarityMetric<T> = Fn(&T, &T) -> Similarity;
