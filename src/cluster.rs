//! This module contains the implementation of grappolo's clustering algorithm.

use std::collections::HashSet;

use crate::{Index, Size};
use crate::sim_matrix::SimilarityMatrix;

type Cluster = Vec<Index>;

/// Result of a clustering run, comprised of one or more `Cluster`s.
pub struct ClusteringResult {
    pub clusters: Vec<Cluster>,
    pub similarity_matrix: SimilarityMatrix,
}

pub struct Clusterer {
    clusters_so_far: Vec<Cluster>,
    visited_so_far: HashSet<Index>,
    current_cluster: Vec<Index>,
}

impl Clusterer {
    /// Cluster a similarity matrix
    ///
    /// # Arguments
    ///
    /// * `similarity_matrix` - Similarity matrix to cluster.
    ///
    /// # Return
    ///
    /// The `Clustering` result.
    pub fn cluster(similarity_matrix: SimilarityMatrix) -> ClusteringResult {
        let mut clusterer = Clusterer {
            clusters_so_far: Vec::new(),
            visited_so_far: HashSet::new(),
            current_cluster: Vec::new(),
        };

        let clusters = clusterer.collect_clusters(&similarity_matrix);

        ClusteringResult { clusters, similarity_matrix }
    }

    fn new() -> Clusterer {
        Clusterer {
            clusters_so_far: Vec::new(),
            visited_so_far: HashSet::new(),
            current_cluster: Vec::new(),
        }
    }

    /// Visit and collect siblings from a given element. Long resulting clusters are recursively split.
    ///
    /// # Arguments
    ///
    /// * `similarity_matrix` - Reference to the similarity matrix to use for traversal.
    ///
    /// # Return
    ///
    /// Collected clusters.
    fn collect_clusters(&mut self, similarity_matrix: &SimilarityMatrix) -> Vec<Cluster> {
        let ranked_indices = similarity_matrix.rank_by_weight();

        for current_index in ranked_indices {
            if self.can_add(current_index) {
                self.new_cluster(current_index);

                let siblings =
                    similarity_matrix[current_index].ranked_siblings(self.to_be_excluded());

                for sibling in siblings {
                    self.add_to_cluster(sibling);
                }

                if self.current_cluster_len() < 3 || self.current_cluster_len() == similarity_matrix.size() {
                    self.commit_current_cluster();
                } else {
                    let similarity_matrix = similarity_matrix.spin_off(&self.current_cluster, 0.0);

                    let mut clusterer = Clusterer::new();
                    let inner_clusters = clusterer.collect_clusters(&similarity_matrix);
                    self.commit_inner_clusters(inner_clusters);
                }
            }
        }

        self.clusters_so_far.clone()
    }

    fn can_add(&self, index: Index) -> bool {
        !self.visited_so_far.contains(&index)
    }

    fn new_cluster(&mut self, index: Index) {
        self.current_cluster = vec![];
        self.add_to_cluster(index);
    }

    fn add_to_cluster(&mut self, index: Index) {
        self.current_cluster.push(index);
        self.visited_so_far.insert(index);
    }

    fn current_cluster_len(&self) -> Size {
        self.current_cluster.len()
    }

    fn commit_current_cluster(&mut self) {
        self.clusters_so_far.push(self.current_cluster.clone());
        self.current_cluster = vec![];
    }

    fn to_be_excluded(&self) -> &HashSet<Index> {
        &self.visited_so_far
    }

    fn commit_inner_clusters(&mut self, sub_clusters: Vec<Cluster>) {
        for sub_cluster in sub_clusters {
            let cluster =
                sub_cluster.iter()
                    .map(|inner_index| self.current_cluster[*inner_index])
                    .collect::<Vec<Index>>();

            self.clusters_so_far.push(cluster);
        }
    }
}

#[cfg(test)]
mod tests {
    use strsim::normalized_damerau_levenshtein;

    use crate::index_pair::cartesian::CartesianIndexPairIterator;
    use crate::utils::*;

    use super::*;

    #[test]
    fn creates_simple_clusters() {
        let names = &string_vec(vec![
            "alejandro", "alejo",
            "martha", "marta",
            "marlene", "marleny", "malrene",
            "ricardo"
        ]);

        let expected_clusters: Vec<Cluster> = vec![
            vec![0, 1, ], // "alejandro", "alejo",
            vec![2, 3, ], // "martha", "marta",
            vec![4, 5, 6, ], // "marlene", "marleny", "malrene",
            vec![7, ], // "ricardo"
        ];

        let similarity_matrix = SimilarityMatrix::new(
            names,
            0.45,
            &mut CartesianIndexPairIterator::new(names.len()),
            |t1, t2| normalized_damerau_levenshtein(t1.as_str(), t2.as_str()),
        );

        let mut clustering = Clusterer::cluster(similarity_matrix);

        clustering.clusters.sort_by(|cluster1, cluster2| cluster1[0].cmp(&cluster2[0]));

        assert_eq!(clustering.clusters, expected_clusters);
    }

    #[test]
    fn creates_recursive_cluster() {
        let min_similarity = 0.7;
        let element_count = 100;
        let expected_cluster_count = 54;

        let names = read_file_lines(String::from("data/surnames.txt"), element_count);

        let expected_element_count = names.len();

        let similarity_matrix = SimilarityMatrix::new(
            &names,
            min_similarity,
            &mut CartesianIndexPairIterator::new(names.len()),
            |t1, t2| normalized_damerau_levenshtein(t1.as_str(), t2.as_str()),
        );

        let clustering = Clusterer::cluster(similarity_matrix);

        assert_eq!(clustering.clusters.len(), expected_cluster_count);

        let actual_element_count = clustering.
            clusters.iter()
            .map(|cluster| cluster.len())
            .sum::<usize>();
        assert_eq!(actual_element_count, expected_element_count);
    }
}
