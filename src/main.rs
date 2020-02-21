use std::io::Write;
use std::time::SystemTime;

use strsim::normalized_damerau_levenshtein;

use grappolo::{Index, Size};
use grappolo::cluster::Clusterer;
use grappolo::index_pair::ngrams::NGramPairs;
use grappolo::sim_matrix::SimilarityMatrix;
use grappolo::utils::*;
use std::fs::OpenOptions;

fn main() {
    show_clusters();
}

fn show_clusters() {
    let min_similarity = 0.75;
    println!("Min similarity: {}", min_similarity);

    let base_filename = "data/surnames";
    let names = {
        let filename = format!("{}.txt", base_filename);
        read_all_file_lines(filename)
    };
    println!("Names: {}", names.len());

    let start_time = SystemTime::now();
    let similarity_matrix = SimilarityMatrix::new(
        &names,
        min_similarity,
        &mut NGramPairs::new(&names, 2),
        |t1, t2| normalized_damerau_levenshtein(t1.as_str(), t2.as_str()),
    );
    println!(
        "Similarity matrix created in {} seconds",
        millis_since(start_time) as f64 / 1000.0);

    let mut out = OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(format!(
            "data/spanish-surnames-matrix-{}.txt",
            min_similarity
        ))
        .expect("Error opening output file");

    for (row_index, row) in similarity_matrix.rows.iter().enumerate() {
        let record =
            row.scores
                .iter()
                .map(|score| {
                    format!("{}/{}/{}", score.sibling_index, names[score.sibling_index], score.similarity)
                })
                .collect::<Vec<String>>()
                .join(", ");
        writeln!(out, "{}/{}: {}", row_index, names[row_index], record)
            .expect("Error writing matrix file");
    }

    println!("Clustering with {} similarity values", &similarity_matrix.similarity_values.len());
    let indices = (0..names.len()).collect::<Vec<Index>>();
    for similarity_value in &similarity_matrix.similarity_values {
        let start_time = SystemTime::now();

        let similarity_matrix = similarity_matrix.spin_off(&indices, *similarity_value);

        let clustering = Clusterer::cluster(similarity_matrix);
        let similarity_value = format!("{0:.2}", similarity_value);
        println!(
            "{} clusters created for similarity {} in {} seconds",
            clustering.clusters.len(),
            similarity_value,
            millis_since(start_time) as f64 / 1000.0);

        let clustered_count = clustering.clusters.iter()
            .map(|cluster| cluster.len())
            .sum::<Size>();
        assert_eq!(clustered_count, names.len());

        let mut out = {
            let filename = format!("{}-clusters-{}.txt", base_filename, similarity_value);
            open_output_file(filename)
        };

        for cluster in clustering.clusters {
            let cluster = &cluster;

            let cluster_names =
                cluster.iter()
                    .map(|index| names[*index].clone())
                    .collect::<Vec<String>>()
                    .join(",");
            write!(out,
                   "{},{}\n",
                   cluster.len(),
                   cluster_names)
                .expect("Error writing cluster file");
        }
        out.flush()
            .expect("Error flushing cluster file");
    }
}
