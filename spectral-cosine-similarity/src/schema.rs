diesel::table! {
    algorithms (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        approximates_algorithm_id -> Nullable<Integer>,
    }
}

diesel::table! {
    libraries (id) {
        id -> Integer,
        name -> Text,
        version -> Text,
        git_commit -> Nullable<Text>,
        git_url -> Nullable<Text>,
        language -> Text,
    }
}

diesel::table! {
    implementations (id) {
        id -> Integer,
        algorithm_id -> Integer,
        library_id -> Integer,
        is_reference -> Bool,
    }
}

diesel::table! {
    experiments (id) {
        id -> Integer,
        params -> Text,
    }
}

diesel::table! {
    molecules (id) {
        id -> Integer,
        smiles -> Text,
        inchikey -> Text,
    }
}

diesel::table! {
    fingerprint_algorithms (id) {
        id -> Integer,
        name -> Text,
        params -> Text,
    }
}

diesel::table! {
    fingerprints (molecule_id, fingerprint_algorithm_id) {
        molecule_id -> Integer,
        fingerprint_algorithm_id -> Integer,
        fingerprint -> Binary,
    }
}

diesel::table! {
    spectra (id) {
        id -> Integer,
        name -> Text,
        raw_name -> Text,
        source_file -> Text,
        spectrum_hash -> Text,
        precursor_mz -> Double,
        num_peaks -> Integer,
        peaks -> Text,
        molecule_id -> Integer,
    }
}

diesel::table! {
    results (left_id, right_id, experiment_id, implementation_id) {
        left_id -> Integer,
        right_id -> Integer,
        experiment_id -> Integer,
        implementation_id -> Integer,
        score -> Double,
        matches -> Integer,
        median_time_us -> Double,
    }
}

diesel::table! {
    selected_pairs (left_id, right_id) {
        left_id -> Integer,
        right_id -> Integer,
    }
}

diesel::table! {
    tanimoto_results (left_id, right_id, fingerprint_algorithm_id) {
        left_id -> Integer,
        right_id -> Integer,
        fingerprint_algorithm_id -> Integer,
        tanimoto_score -> Double,
    }
}

diesel::joinable!(implementations -> algorithms (algorithm_id));
diesel::joinable!(implementations -> libraries (library_id));
diesel::joinable!(results -> experiments (experiment_id));
diesel::joinable!(results -> implementations (implementation_id));
diesel::joinable!(spectra -> molecules (molecule_id));
diesel::joinable!(fingerprints -> molecules (molecule_id));
diesel::joinable!(fingerprints -> fingerprint_algorithms (fingerprint_algorithm_id));
diesel::joinable!(tanimoto_results -> fingerprint_algorithms (fingerprint_algorithm_id));

diesel::allow_tables_to_appear_in_same_query!(
    algorithms,
    libraries,
    implementations,
    experiments,
    molecules,
    fingerprint_algorithms,
    fingerprints,
    spectra,
    results,
    selected_pairs,
    tanimoto_results,
);
