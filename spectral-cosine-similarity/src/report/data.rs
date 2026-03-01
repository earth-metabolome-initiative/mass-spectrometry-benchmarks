use std::collections::HashMap;

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel::sqlite::SqliteConnection;

use crate::schema::{algorithms, implementations, libraries, results, spectra};

use super::style::series_label;
use super::types::{AlgorithmReference, ResultRow};

#[derive(QueryableByName)]
struct CanonicalAlgorithmRow {
    #[diesel(sql_type = Text)]
    algorithm_name: String,
    #[diesel(sql_type = Text)]
    canonical_algorithm_name: String,
}

pub(crate) fn load_spectra_peaks(conn: &mut SqliteConnection) -> HashMap<i32, i32> {
    spectra::table
        .order(spectra::id.asc())
        .select((spectra::id, spectra::num_peaks))
        .load::<(i32, i32)>(conn)
        .expect("failed to load spectra peaks")
        .into_iter()
        .collect()
}

pub(crate) fn load_result_data(conn: &mut SqliteConnection) -> Vec<ResultRow> {
    results::table
        .inner_join(
            implementations::table
                .inner_join(algorithms::table)
                .inner_join(libraries::table),
        )
        .order((
            algorithms::name.asc(),
            libraries::name.asc(),
            results::experiment_id.asc(),
            results::left_id.asc(),
            results::right_id.asc(),
        ))
        .select((
            implementations::id,
            implementations::is_reference,
            results::score,
            results::median_time_us,
            algorithms::name,
            libraries::name,
            results::left_id,
            results::right_id,
            results::experiment_id,
        ))
        .load::<(i32, bool, f32, f32, String, String, i32, i32, i32)>(conn)
        .expect("failed to load result data")
        .into_iter()
        .map(
            |(
                implementation_id,
                is_reference,
                score,
                median_time_us,
                algo_name,
                lib_name,
                left_id,
                right_id,
                experiment_id,
            )| ResultRow {
                implementation_id,
                is_reference,
                score,
                median_time_us,
                algo_name,
                lib_name,
                left_id,
                right_id,
                experiment_id,
            },
        )
        .collect()
}

pub(crate) fn load_algorithm_canonical_map(conn: &mut SqliteConnection) -> HashMap<String, String> {
    let rows: Vec<CanonicalAlgorithmRow> = sql_query(
        "SELECT child.name AS algorithm_name,
                COALESCE(parent.name, child.name) AS canonical_algorithm_name
         FROM algorithms child
         LEFT JOIN algorithms parent ON child.approximates_algorithm_id = parent.id
         ORDER BY child.id",
    )
    .load(conn)
    .expect("failed to load canonical algorithm map");

    rows.into_iter()
        .map(|r| (r.algorithm_name, r.canonical_algorithm_name))
        .collect()
}

pub(crate) fn algorithm_references(
    data: &[ResultRow],
    canonical_algorithm_map: &HashMap<String, String>,
) -> HashMap<String, AlgorithmReference> {
    let mut canonical_refs: HashMap<String, AlgorithmReference> = HashMap::new();

    for row in data.iter().filter(|r| r.is_reference) {
        let canonical_name = canonical_algorithm_map
            .get(&row.algo_name)
            .map(String::as_str)
            .unwrap_or(&row.algo_name);

        // Only canonical algorithms define canonical references.
        if row.algo_name != canonical_name {
            continue;
        }

        let label = series_label(&row.algo_name, &row.lib_name);
        match canonical_refs.entry(canonical_name.to_string()) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(AlgorithmReference {
                    implementation_id: row.implementation_id,
                    label,
                });
            }
            std::collections::hash_map::Entry::Occupied(entry) => {
                if entry.get().implementation_id != row.implementation_id {
                    panic!(
                        "canonical algorithm '{}' has multiple reference implementations in results",
                        canonical_name
                    );
                }
            }
        }
    }

    let mut refs: HashMap<String, AlgorithmReference> = HashMap::new();
    let mut algorithms: Vec<String> = data.iter().map(|r| r.algo_name.clone()).collect();
    algorithms.sort();
    algorithms.dedup();

    for algorithm_name in algorithms {
        let canonical_name = canonical_algorithm_map
            .get(&algorithm_name)
            .map(String::as_str)
            .unwrap_or(&algorithm_name);

        if let Some(reference) = canonical_refs.get(canonical_name) {
            refs.insert(algorithm_name, reference.clone());
        }
    }

    refs
}
