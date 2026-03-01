use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Bool, Integer, Nullable, Text};
use diesel::sqlite::SqliteConnection;

#[derive(Clone, Debug, QueryableByName, PartialEq, Eq)]
pub struct ResolvedImplementationRow {
    #[diesel(sql_type = Integer)]
    pub implementation_id: i32,
    #[diesel(sql_type = Text)]
    pub algorithm_name: String,
    #[diesel(sql_type = Text)]
    pub canonical_algorithm_name: String,
    #[diesel(sql_type = Text)]
    pub library_name: String,
    #[diesel(sql_type = Text)]
    pub library_language: String,
    #[diesel(sql_type = Bool)]
    pub is_reference: bool,
    #[diesel(sql_type = Nullable<Integer>)]
    pub canonical_reference_implementation_id: Option<i32>,
    #[diesel(sql_type = Nullable<Text>)]
    pub canonical_reference_library_name: Option<String>,
}

pub fn load_resolved_implementations(
    conn: &mut SqliteConnection,
) -> Vec<ResolvedImplementationRow> {
    sql_query(
        "SELECT implementation_id,
                algorithm_name,
                canonical_algorithm_name,
                library_name,
                library_language,
                is_reference,
                canonical_reference_implementation_id,
                canonical_reference_library_name
         FROM v_implementation_topology
         ORDER BY algorithm_name, library_name, implementation_id",
    )
    .load(conn)
    .expect("failed to load resolved implementation topology")
}

#[cfg(test)]
mod tests {
    use diesel::Connection;

    use super::*;
    use crate::db;

    fn setup_in_memory_connection() -> SqliteConnection {
        SqliteConnection::establish(":memory:").expect("failed to open in-memory sqlite db")
    }

    #[test]
    fn resolves_canonical_algorithm_names_for_approximations() {
        let mut conn = setup_in_memory_connection();
        db::initialize(&mut conn);

        let rows = load_resolved_implementations(&mut conn);

        let cosine_greedy = rows
            .iter()
            .find(|row| {
                row.algorithm_name == "CosineGreedy"
                    && row.library_name == "mass-spectrometry-traits"
            })
            .expect("missing CosineGreedy implementation row");
        assert_eq!(cosine_greedy.canonical_algorithm_name, "CosineHungarian");

        let modified_greedy = rows
            .iter()
            .find(|row| {
                row.algorithm_name == "ModifiedGreedyCosine"
                    && row.library_name == "mass-spectrometry-traits"
            })
            .expect("missing ModifiedGreedyCosine implementation row");
        assert_eq!(modified_greedy.canonical_algorithm_name, "ModifiedCosine");
    }

    #[test]
    fn resolves_canonical_reference_implementations_for_algorithm_families() {
        let mut conn = setup_in_memory_connection();
        db::initialize(&mut conn);

        let expected_cosine_ref =
            db::get_implementation_id(&mut conn, "CosineHungarian", "matchms");
        let expected_modified_ref =
            db::get_implementation_id(&mut conn, "ModifiedCosine", "mass-spectrometry-traits");

        let rows = load_resolved_implementations(&mut conn);

        for algorithm in ["CosineHungarian", "CosineGreedy"] {
            let row = rows
                .iter()
                .find(|r| r.algorithm_name == algorithm && r.library_name == "matchms")
                .unwrap_or_else(|| panic!("missing row for algorithm {algorithm}"));
            assert_eq!(
                row.canonical_reference_implementation_id,
                Some(expected_cosine_ref)
            );
            assert_eq!(
                row.canonical_reference_library_name.as_deref(),
                Some("matchms")
            );
        }

        for algorithm in ["ModifiedCosine", "ModifiedGreedyCosine"] {
            let row = rows
                .iter()
                .find(|r| r.algorithm_name == algorithm)
                .unwrap_or_else(|| panic!("missing row for algorithm {algorithm}"));
            assert_eq!(
                row.canonical_reference_implementation_id,
                Some(expected_modified_ref)
            );
            assert_eq!(
                row.canonical_reference_library_name.as_deref(),
                Some("mass-spectrometry-traits")
            );
        }
    }
}
