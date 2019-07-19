//! This module contains clustering evaluation metrics used to compare clustering quality.

use crate::cluster::ClusteringResult;

/// The `f64` type for cluster evaluation.
type ClusterEvaluation = f64;

/// A metric for measuring the quality of a `Clustering` result.
pub trait ClusterEvaluator {
    /// Compare two `Clustering` results to measure which one is "best" according to this evaluator.
    ///
    /// # Arguments
    ///
    /// `clustering` - The `Clustering` result to be evaluated.
    fn evaluate(clustering: &ClusteringResult) ->  ClusterEvaluation;
    /// Ascertain whether a given `ClusterEvaluation` is "better" than another.
    ///
    /// # Arguments
    ///
    /// * `e1` - The first evaluation value.
    /// * `e2` - The second evaluation value.
    ///
    /// # Return
    ///
    /// A boolean value indicating whether `e1` is a better value than `e2`.
    fn best_of(e1: ClusterEvaluation, e2: ClusterEvaluation) -> bool;
}